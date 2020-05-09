use obj::*;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let input = BufReader::new(File::open("resources/navmesh.obj").unwrap());
    let obj: Obj<Position> = load_obj(input).unwrap();
    // let dome: Obj = load_obj(input).unwrap();

    for index in obj.indices {
        let vertex = obj.vertices[index as usize];
        println!("{:?} {:?}", index, vertex);
    }
    // println!("{:?}", obj);
    // Do whatever you want
    // dome.vertices;
    // dome.indices;
}
