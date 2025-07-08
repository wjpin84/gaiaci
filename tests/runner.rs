#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn runs_simple_pipeline() {
        let dir = tempdir().unwrap();
        let gaiaci_dir = dir.path().join(".gaiaci");
        fs::create_dir(&gaiaci_dir).unwrap();

        let pipeline = r#"
          return {
            jobs = {
              test_job = {
                steps = {
                  { run = "echo Hello from Lua job" },
                   { 
                }
              }
            }
          }
        "#;

        fs::write(gaiaci_dir.join("pipeline.lua"), pipeline).unwrap();

        Command::cargo_bin("gaia")
            .unwrap()
            .current_dir(dir.path())
            .arg("run")
            .assert()
            .success()
            .stdout(predicates::str::contains("Hello from Lua job"));
    }
}
