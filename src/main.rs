fn main() -> std::io::Result<()> {
    let archive = {
        let file = std::fs::File::open("corpus/msa.tpxz")?;
        liblzma::decode_all(file).unwrap().into_boxed_slice()
    };
    let edges = {
        let mut map = std::collections::BTreeMap::<String, Vec<(String, &[u8])>>::new();
        for file in tar::Archive::new(&*archive).entries().unwrap() {
            let file = file.unwrap();
            let path = file.path().unwrap();
            let src = path.parent().unwrap().to_str().unwrap().to_string();
            let dst = path.file_stem().unwrap().to_str().unwrap().to_string();
            let file_pos = file.raw_file_position() as usize;
            let entry_size = file.size() as usize;
            let slice = &archive[file_pos..][..entry_size];
            if !map.contains_key(&src) {
                map.insert(src.clone(), Vec::new());
            }
            if let Some(vec) = map.get_mut(&src) {
                vec.push((dst.clone(), slice));
            }
        }
        map
    };
    let mut stack = Vec::new();
    let empty = Vec::new();
    let root = edges
        .get("0000000000000000000000000000000000000000000000000000000000000000-0")
        .unwrap();
    stack.push((empty, root.iter()));
    while !stack.is_empty() {
        let len = stack.len();
        let next_edge = stack[len - 1].1.next().unwrap();
        let mut raw = lz4_flex::block::decompress(next_edge.1, 3 << 17).unwrap();
        if len > 1 {
            let mut encoded = Vec::new();
            aehobak::encode(&raw, &mut encoded).unwrap();
            let compressed = lz4_flex::block::compress(&encoded);
            let preimage = &stack[len - 1].0;
            let mut postimage = Vec::new();
            bsdiff::patch(preimage, &mut raw.as_slice(), &mut postimage)?;
            let post_lz4 = lz4_flex::block::compress(&postimage);
            println!(
                "{} {} {} {} {} {}",
                postimage.len(),
                post_lz4.len(),
                raw.len(),
                next_edge.1.len(),
                encoded.len(),
                compressed.len()
            );
            raw = postimage;
        }
        if let Some(next) = edges.get(&next_edge.0) {
            stack.push((raw, next.iter()));
        } else {
            while stack
                .last()
                .map(|(_, iter)| iter.len() == 0)
                .unwrap_or_default()
            {
                stack.pop();
            }
        }
    }
    Ok(())
}
