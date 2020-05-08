use obj::*;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let input = BufReader::new(File::open("resources/navmesh.obj").unwrap());
    let dome: Obj = load_obj(input).unwrap();

    for index in dome.indices {
        let vertex = dome.vertices[index as usize];
        println!("{:?} {:?}", index, vertex);
    }
    // Do whatever you want
    // dome.vertices;
    // dome.indices;
}
