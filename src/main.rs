use crypt_ui::cli::load_cli;
fn main() -> anyhow::Result<()> {
    load_cli();

    // _ = tester();

    Ok(())
}

// fn tester() {
//     // Example PathBuf
//     let path_buf =
// PathBuf::from("C:\\Users\\ryanm\\crypt\\test_folder\\folder2\\New folder\\apples.crypt");

//     // Find the position of "crypt" in the path
//     let crypt_position = path_buf.iter().position(|component| component == "crypt");

//     if let Some(index) = crypt_position {
//         // Collect the components after "crypt"
//         let remaining_components: Vec<_> = path_buf.iter().skip(index + 1).collect();

//         let len = remaining_components.len() - 1;

//         // Check if there are remaining components
//         if remaining_components.is_empty() {
//             println!("No remaining components after 'crypt'.");
//             return;
//         }

//         // Iterate over each remaining component
//         for (num, component) in remaining_components.iter().enumerate() {
//             if num != len {
//                 println!("directory: {:?}", component);
//             }
//             if num == len {
//                 println!("file! : {:?}", component);
//             }
//         }
//     } else {
//         println!("unable to parse file path. are we in the crypt folder?");
//     }
// }
