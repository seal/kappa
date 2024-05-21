#[cfg(test)]
mod tests {
    use std::fs;

    use super::super::docker::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_delete_docker_container_and_image() {
        let container_id = Uuid::new_v4();
        let result = delete_docker_container_and_image(&container_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dockerise_container() {
        let container_id = Uuid::new_v4();
        let zip_path = format!("./zip/{}", container_id);
        let main_go_path = format!("{}/main.go", zip_path);

        // Create the directory and main.go file
        fs::create_dir_all(&zip_path).unwrap();
        let main_go_content = r#"
        package main

        import "fmt"

        func main() {
            fmt.Println("Hello, World!")
        }
    "#;
        fs::write(&main_go_path, main_go_content).unwrap();

        let result = dockerise_container(container_id).await;
        assert!(result.is_ok());
        let port = result.unwrap();
        assert!(port >= 5000 && port < 6000);

        // Clean up the created directory and file
        //  Cleaup is done in delete function
        /*
        fs::remove_file(main_go_path).unwrap();
        fs::remove_dir_all(zip_path).unwrap();
        */
        let result = delete_docker_container_and_image(&container_id).await;
        assert!(result.is_ok());
    }
}
