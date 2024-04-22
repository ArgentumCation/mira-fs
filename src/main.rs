use std::{ffi::OsStr, num::NonZeroU32, vec::IntoIter};

use fuse3::raw::{
    reply::{DirectoryEntry, DirectoryEntryPlus, ReplyInit},
    Request,
};
use futures_util::stream::{Empty, Iter};
use indradb::{self, ijson, Edge, Identifier, SpecificVertexQuery, Vertex};
use indradb::{Database, MemoryDatastore};
use walkdir::WalkDir;
mod snowflake;

struct Filesystem {}

impl fuse3::raw::Filesystem for Filesystem {
    type DirEntryStream<'a> = Empty<fuse3::Result<DirectoryEntry>> where Self: 'a;
    type DirEntryPlusStream<'a> = Iter<IntoIter<fuse3::Result<DirectoryEntryPlus>>> where Self: 'a;
    async fn init(&self, _req: Request) -> fuse3::Result<ReplyInit> {
        Ok(ReplyInit {
            max_write: NonZeroU32::new(16 * 1024).unwrap(),
        })
    }

    async fn destroy(&self, _req: Request) {}
}

fn db_sync(db: &Database<MemoryDatastore>) {
    let tmpdir = std::env::var("TMPDIR").ok();
    std::env::set_var("TMPDIR", std::env::current_dir().unwrap());
    db.sync().unwrap();
    std::env::set_var("TMPDIR", tmpdir.unwrap_or(String::new()));
}
fn main() {
    // TODO: override tmpdir
    let db = MemoryDatastore::create_msgpack_db(std::env::current_dir().unwrap().join("owo.db"));
    let folder = "/home/akristip/20 Personal/Lunar-Witch";
    let stack = &mut WalkDir::new(folder)
        .max_depth(0)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();
    // let mut parent = OsString::from("root");
    let mut node = Vertex::new(Identifier::new("fs").unwrap());
    db.create_vertex(&node).unwrap();
    db.set_properties(
        SpecificVertexQuery::new(vec![node.id]),
        Identifier::new("name").unwrap(),
        &ijson!("root"),
    )
    .unwrap();
    while let Some(entry) = stack.pop() {
        print!("Parent: {:#?}\t ", &node);
        if entry.file_type().is_dir() {
            println!("Dir: {:#?}", &entry.file_name());
            let new_node = Vertex::new(Identifier::new("fs").unwrap());
            db.create_vertex(&new_node).unwrap();
            db.set_properties(
                SpecificVertexQuery::new(vec![new_node.id]),
                Identifier::new("name").unwrap(),
                &ijson!(entry.file_name().to_string_lossy()),
            )
            .unwrap();
            db.create_edge(&Edge::new(
                node.id,
                Identifier::new("parent-of").unwrap(),
                new_node.id,
            ))
            .unwrap();
            // dbg!(&db.datastore);
            node = new_node;
            // parent = entry.file_name().into();
            let child_dirs = walkdir::WalkDir::new(entry.path())
                .min_depth(1)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok());
            stack.extend(child_dirs);
        } else {
            println!("File: {:#?}", entry.path());
        }
        // println!(
        //     "{:#?}\t {:#?}\t {:#?}",
        //     entry.path().into_iter().collect::<Vec<&OsStr>>(),
        //     entry.file_name(),
        //     entry.depth()
        // );
    }
    db_sync(&db)
}
