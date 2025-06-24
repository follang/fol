// Comprehensive tests for fol-stream module

use fol_stream::{FileStream, CharacterProvider, Source, SourceType};

// Include sophisticated .mod handling tests
mod test_mod_handling;

// Include namespace tests
mod test_namespace;

#[cfg(test)]
mod stream_tests {
    use super::*;

    #[test]
    fn test_basic_file_reading() {
        let mut stream = FileStream::from_file("test/stream/basic.fol")
            .expect("Should be able to read basic.fol");
        
        let expected = "hello world".chars().collect::<Vec<_>>();
        let mut actual = Vec::new();
        
        while let Some((ch, _loc)) = stream.next_char() {
            actual.push(ch);
        }
        
        assert_eq!(actual, expected, "Should read file contents correctly");
    }

    #[test]
    fn test_location_tracking_single_line() {
        let mut stream = FileStream::from_file("test/stream/basic.fol")
            .expect("Should be able to read basic.fol");
        
        // Test first character location
        if let Some((ch, loc)) = stream.next_char() {
            assert_eq!(ch, 'h');
            assert_eq!(loc.row, 1);
            assert_eq!(loc.col, 1);
            assert_eq!(loc.file, Some("test/stream/basic.fol".to_string()));
        } else {
            panic!("Should have at least one character");
        }
        
        // Test a few more characters
        if let Some((ch, loc)) = stream.next_char() {
            assert_eq!(ch, 'e');
            assert_eq!(loc.row, 1);
            assert_eq!(loc.col, 2);
        }
        
        // Skip to space
        for _ in 0..3 { stream.next_char(); } // l, l, o
        if let Some((ch, loc)) = stream.next_char() {
            assert_eq!(ch, ' ');
            assert_eq!(loc.row, 1);
            assert_eq!(loc.col, 6);
        }
    }

    #[test]
    fn test_multiline_location_tracking() {
        let mut stream = FileStream::from_file("test/stream/multiline.fol")
            .expect("Should be able to read multiline.fol");
        
        let mut chars_and_locations = Vec::new();
        while let Some((ch, loc)) = stream.next_char() {
            chars_and_locations.push((ch, loc.clone()));
        }
        
        // Verify we have the right content
        let chars: String = chars_and_locations.iter().map(|(ch, _)| *ch).collect();
        assert_eq!(chars, "line 1\nline 2\nline 3");
        
        // Check specific locations
        // First character of line 1
        assert_eq!(chars_and_locations[0].1.row, 1);
        assert_eq!(chars_and_locations[0].1.col, 1);
        
        // Newline at end of line 1 (position 6, 0-indexed)
        assert_eq!(chars_and_locations[6].0, '\n');
        assert_eq!(chars_and_locations[6].1.row, 1);
        assert_eq!(chars_and_locations[6].1.col, 7);
        
        // First character of line 2 (position 7)
        assert_eq!(chars_and_locations[7].0, 'l');
        assert_eq!(chars_and_locations[7].1.row, 2);
        assert_eq!(chars_and_locations[7].1.col, 1);
        
        // First character of line 3 (position 14)
        assert_eq!(chars_and_locations[14].0, 'l');
        assert_eq!(chars_and_locations[14].1.row, 3);
        assert_eq!(chars_and_locations[14].1.col, 1);
    }

    #[test]
    fn test_unicode_handling() {
        let mut stream = FileStream::from_file("test/stream/unicode.fol")
            .expect("Should be able to read unicode.fol");
        
        let mut chars = Vec::new();
        while let Some((ch, _loc)) = stream.next_char() {
            chars.push(ch);
        }
        
        let content: String = chars.iter().collect();
        assert!(content.contains("世界"), "Should handle Chinese characters");
        assert!(content.contains("🌍"), "Should handle emoji");
        assert!(content.contains("café"), "Should handle accented characters");
        assert!(content.contains("résumé"), "Should handle multiple accents");
        assert!(content.contains("naïve"), "Should handle diaeresis");
    }

    #[test]
    fn test_whitespace_handling() {
        let mut stream = FileStream::from_file("test/stream/whitespace.fol")
            .expect("Should be able to read whitespace.fol");
        
        let mut chars_and_locations = Vec::new();
        while let Some((ch, loc)) = stream.next_char() {
            chars_and_locations.push((ch, loc.clone()));
        }
        
        // Check that tabs and spaces are preserved
        assert_eq!(chars_and_locations[0].0, '\t', "Should preserve leading tab");
        
        // Find spaces in content
        let space_positions: Vec<_> = chars_and_locations.iter()
            .enumerate()
            .filter(|(_, (ch, _))| *ch == ' ')
            .collect();
        
        assert!(!space_positions.is_empty(), "Should have spaces in the file");
        
        // Verify location tracking works with tabs and spaces
        for (pos, (ch, loc)) in chars_and_locations.iter().enumerate() {
            if *ch == '\n' {
                // Next character should be on new line
                if pos + 1 < chars_and_locations.len() {
                    assert_eq!(chars_and_locations[pos + 1].1.row, loc.row + 1);
                    assert_eq!(chars_and_locations[pos + 1].1.col, 1);
                }
            }
        }
    }

    #[test]
    fn test_empty_file() {
        let mut stream = FileStream::from_file("test/stream/empty.fol")
            .expect("Should be able to read empty.fol");
        
        let result = stream.next_char();
        assert!(result.is_none(), "Empty file should return None immediately");
    }

    #[test]
    fn test_nonexistent_file() {
        let result = FileStream::from_file("test/stream/nonexistent.fol");
        assert!(result.is_err(), "Should return error for nonexistent file");
    }

    #[test]
    fn test_file_path_preservation() {
        let file_path = "test/stream/basic.fol";
        let mut stream = FileStream::from_file(file_path)
            .expect("Should be able to read file");
        
        if let Some((_ch, loc)) = stream.next_char() {
            assert_eq!(loc.file, Some(file_path.to_string()), "Should preserve file path");
        }
    }

    #[test]
    fn test_complete_file_processing() {
        let mut stream = FileStream::from_file("test/stream/multiline.fol")
            .expect("Should be able to read multiline.fol");
        
        let mut char_count = 0;
        let mut line_count = 0;
        let mut max_col = 0;
        
        while let Some((ch, loc)) = stream.next_char() {
            char_count += 1;
            line_count = line_count.max(loc.row);
            max_col = max_col.max(loc.col);
            
            if ch == '\n' {
                // Column should reset after newline
                if let Some((_, next_loc)) = stream.next_char() {
                    if next_loc.col != 1 {
                        panic!("Column should reset to 1 after newline, got {}", next_loc.col);
                    }
                    // Put the character back by decrementing position manually
                    // (This is a limitation of our simple test, in real usage we'd peek)
                }
            }
        }
        
        assert_eq!(char_count, 20, "Should count all characters including newlines"); // "line 1\nline 2\nline 3" = 20 chars
        assert_eq!(line_count, 3, "Should track all three lines");
        assert!(max_col >= 6, "Should track column positions correctly");
    }
}

// Performance and stress tests
#[cfg(test)]
mod stream_performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_large_file_performance() {
        // Create a larger test file
        use std::fs::File;
        use std::io::Write;
        
        let large_file_path = "test/stream/large_test.fol";
        {
            let mut file = File::create(large_file_path).expect("Should create test file");
            for i in 0..1000 {
                writeln!(file, "line {} with some content to make it longer", i)
                    .expect("Should write line");
            }
        }
        
        let start = Instant::now();
        let mut stream = FileStream::from_file(large_file_path)
            .expect("Should read large file");
        
        let mut char_count = 0;
        while let Some((_ch, _loc)) = stream.next_char() {
            char_count += 1;
        }
        
        let duration = start.elapsed();
        
        // Clean up
        std::fs::remove_file(large_file_path).ok();
        
        println!("Processed {} characters in {:?}", char_count, duration);
        assert!(char_count > 40000, "Should process substantial number of characters");
        assert!(duration.as_secs() < 1, "Should process large file reasonably quickly");
    }
}