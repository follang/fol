// Test workspace integration

use fol_types::*;
use fol_stream::*;

fn main() {
    println!("Testing FOL workspace integration...");
    
    // Test fol-types
    let result: Con<i32> = Ok(42);
    println!("✅ fol-types: {:?}", result);
    
    // Test fol-stream
    let test_content = "hello world";
    std::fs::write("test_file.txt", test_content).unwrap();
    
    match FileStream::from_file("test_file.txt") {
        Ok(mut stream) => {
            println!("✅ fol-stream: Created file stream");
            if let Some((ch, loc)) = stream.next_char() {
                println!("   First char: '{}' at {}:{}", ch, loc.row, loc.col);
            }
        }
        Err(e) => println!("❌ fol-stream error: {}", e),
    }
    
    // Cleanup
    std::fs::remove_file("test_file.txt").ok();
    
    println!("✅ Workspace integration test complete!");
}