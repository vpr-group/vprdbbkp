#[cfg(test)]
mod provider_test {
    use crate::test_utils::test_utils::{get_local_provider, get_s3_provider, initialize_test};
    use std::io::{Cursor, Read, Write};

    #[tokio::test]
    async fn test_01_local() {
        initialize_test();
        let provider = get_local_provider().expect("Failed to get local provider");
        let content = "Ceci est un message test".as_bytes();

        let mut content_reader = Cursor::new(content);
        let mut writer = provider
            .create_writer("test")
            .await
            .expect("Failed to create writer");

        let mut buffer = [0u8; 10];

        loop {
            match content_reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    writer
                        .write(&buffer[..n].to_vec())
                        .expect("Failed to write bytes");
                }
                Err(e) => panic!("Error while reading content: {}", e),
            }
        }

        writer.flush().expect("Failed to flush the writer");

        let is_connected = provider.test().await.expect("Failed to test the provider");
        assert!(is_connected);

        let mut reader = provider
            .create_reader("test")
            .await
            .expect("Failed to create reader");

        let mut reader_content = vec![];

        while let Ok(n) = reader.read(&mut buffer) {
            if (n) > 0 {
                reader_content.extend_from_slice(&buffer[..n]);
            } else {
                break;
            }
        }

        assert_eq!(reader_content.as_slice(), content);

        let entries = provider.list().await.expect("Failed to list entries");
        let first_entry = entries.first().expect("No entry found");
        assert!(first_entry.metadata.content_length > 0);
    }

    #[tokio::test]
    async fn test_02_s3() {
        initialize_test();
        let provider = get_s3_provider().expect("Failed to get s3 provider");

        let mut writer = provider
            .create_writer("test-01")
            .await
            .expect("Failed to create writer");

        let content = "Ceci est un message test".as_bytes();
        let mut reader = Cursor::new(content);
        let mut buffer = [0u8; 10];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    writer.write(&buffer[..n]).expect("Failed to write bytes");
                }
                Err(e) => panic!("Error while reading content: {}", e),
            }
        }

        writer.flush().expect("Failed to flush");

        let mut reader = provider
            .create_reader("test-01")
            .await
            .expect("Failed to create reader");

        let mut reader_content = vec![];

        while let Ok(n) = reader.read(&mut buffer) {
            if (n) > 0 {
                reader_content.extend_from_slice(&buffer[..n]);
            } else {
                break;
            }
        }

        assert_eq!(content, reader_content.as_slice());
    }

    #[tokio::test]
    async fn test_03_list_local() {
        let provider = get_local_provider().expect("Failed to get s3 provider");

        let entries = provider
            .operator
            .list_with("/")
            .recursive(true)
            .limit(10)
            .await
            .expect("Failed to list dumps in");

        println!("{:?}", entries);
    }

    #[tokio::test]
    async fn test_03_list_s3() {
        let provider = get_s3_provider().expect("Failed to get s3 provider");

        let entries = provider
            .operator
            .list_with("/")
            .recursive(true)
            .limit(10)
            .await
            .expect("Failed to list dumps in");

        println!("{:?}", entries);
    }
}
