use std::{env::args, fs::File};

use camino::Utf8PathBuf;
use png::{Decoder, Encoder};

fn main() {
	// usage: in out wrap_count
	let in_file = Utf8PathBuf::from(std::env::args().nth(1).unwrap());
	let out_file = Utf8PathBuf::from(std::env::args().nth(2).unwrap());
	let wraps: usize = std::env::args().nth(3).unwrap().parse().unwrap();

	if !in_file.exists() {
		eprintln!("{in_file} doesn't exist!");
		return;
	}

	let dec = Decoder::new(File::open(in_file).unwrap());
	let mut reader = dec.read_info().unwrap();
	let mut data = vec![0; reader.output_buffer_size()];
	let info = reader.next_frame(&mut data).unwrap();
	let width = info.width as usize;
	let height = info.height as usize;
	let color = info.color_type;
	// Assumes 1 byte per sample
	let samples = info.color_type.samples();

	let image = Image {
		data,
		width,
		height,
	};

	println!("Image! {width}x{height} [{color:?}: {samples}]");

	let wrapped_width = image.width / wraps;
	let wrapped_height = image.height * wraps;

	let mut out_image = Image {
		data: vec![0; wrapped_width * wrapped_height * samples],
		width: wrapped_width,
		height: wrapped_height,
	};

	// yeah let's do it weird and straight indexed. Why? I don't know. Why not. It's bad code, embrace it!
	for dest_idx in 0..wrapped_width * wrapped_height {
		let dest_x = dest_idx % wrapped_width;
		let dest_y = dest_idx / wrapped_width;
		let wrap_row = dest_y / height;

		let source_x = wrap_row * wrapped_width + dest_x;
		let source_y = dest_y - (wrap_row * height);
		let source_idx = source_y * width + source_x;

		// I *cannot* remember what the memcopy is here so this... this is something! it's faster
		// than looking for it and I'm time contrained
		for samp in 0..samples {
			out_image.data[dest_idx * samples + samp] = image.data[source_idx * samples + samp];
		}
	}

	let ofile = File::create(out_file).unwrap();
	let mut enc = Encoder::new(ofile, out_image.width as u32, out_image.height as u32);
	enc.set_color(info.color_type);
	enc.set_depth(info.bit_depth);
	let mut write = enc.write_header().unwrap();
	write.write_image_data(&out_image.data).unwrap()
}

struct Image {
	data: Vec<u8>,
	width: usize,
	height: usize,
}
