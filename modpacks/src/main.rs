mod mrpack;

fn main() {
    let index = mrpack::modrinth_index::ModrinthIndex::deserialize(include_str!(
        "../../test_modpack/modpack_modrinthtype/modrinth.index.json"
    ))
    .unwrap();
    index.files_to_dl().start();
}
