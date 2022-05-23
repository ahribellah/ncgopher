use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::fs::File as FsFile;
use std::io::Write;
use std::path::PathBuf;
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bookmark {
    pub title: String,
    pub url: Url,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Bookmarks {
    /// All bookmarks
    pub entries: Vec<Bookmark>,
}

impl Bookmarks {
    pub fn new() -> Bookmarks {
        let confdir = Bookmarks::get_bookmark_path();
        println!("Looking for bookmarks file {:?}", confdir);
        let mut bookmarks_string = String::new();
        if confdir.as_path().exists() {
            bookmarks_string = read_to_string(confdir).unwrap();
        }
        println!("Reading bookmarks...");
        let bookmarks_table: HashMap<String, Vec<Bookmark>> =
            toml::from_str(&bookmarks_string).unwrap_or_default();
        let entries: &[Bookmark] = match bookmarks_table.contains_key("bookmark") {
            true => &bookmarks_table["bookmark"],
            false => &[],
        };

        Bookmarks {
            entries: entries.to_vec(),
        }
    }

    fn get_bookmark_path() -> PathBuf {
        let mut dir = dirs::config_dir().expect("no configuration directory");
        dir.push(env!("CARGO_PKG_NAME"));
        dir.push("bookmarks");
        info!("Looking for bookmark file {:?}", dir);
        dir
    }

    /// Replace an existting bookmark or add a new bookmark.
    /// If an entry is replaced, it will remain at the same position
    /// Returns the index of the existing entry or None.
    pub fn insert(&mut self, entry: Bookmark) -> Option<usize> {
        info!("Adding entry to bookmark: {:?}", entry);
        let index = self.entries.iter().position(|e| e.url == entry.url);
        if let Some(i) = index {
            // replace item
            self.entries.remove(i);
            self.entries.insert(i, entry);
        } else {
            // insert new item at end
            self.entries.push(entry);
        };
        self.write_bookmarks_to_file()
            .unwrap_or_else(|err| warn!("Could not write bookmarks file: {}", err));
        index
    }

    pub fn remove(&mut self, url: &Url) {
        info!("Removing entry to bookmark: {:?}", url);
        self.entries.retain(|e| &e.url != url);
        if let Err(why) = self.write_bookmarks_to_file() {
            warn!("Could not write bookmarks file: {}", why)
        }
    }

    pub fn get_bookmarks(&self) -> Vec<Bookmark> {
        self.entries.clone()
    }

    pub fn write_bookmarks_to_file(&mut self) -> std::io::Result<()> {
        let path = Bookmarks::get_bookmark_path();
        info!("Saving bookmarks to file: {:?}", path);

        let mut file = match FsFile::create(&path) {
            Err(why) => return Err(why),
            Ok(file) => file,
        };

        file.write_all(b"# Automatically generated by ncgopher.\n")?;
        for b in self.clone().entries {
            file.write_all(b"\n[[bookmark]]\n")?;
            let item = toml::to_string(&b).unwrap();
            file.write_all(item.as_bytes())?;
        }
        Ok(())
    }
}
