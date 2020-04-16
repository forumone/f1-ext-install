use bollard::{
    image::{BuildImageOptions, BuildImageResults},
    Docker,
};
use futures::StreamExt as _;
use std::convert::TryInto as _;
use tar::{Builder, Header};
use tokio_test::block_on;

/// Path to the `f1-ext-install` binary.
pub const F1_EXT_INSTALL_PATH: &str = "target/x86_64-unknown-linux-musl/debug/f1-ext-install";

/// PHP versions to run integration tests against.
pub const PHP_VERSIONS: &[&str] = &["7.3", "7.4"];

/// Create a Docker client that is connected to the local daemon.
pub fn connect() -> Docker {
    let client = Docker::connect_with_local_defaults().unwrap();
    block_on(client.negotiate_version()).unwrap()
}

/// Performs a Docker image build as an integration test
pub fn build_image(client: &Docker, dockerfile: &str, args: &[(&str, &str)], tag: &str) {
    // Create a tar file for the Docker build context, adding f1-ext-install from disk
    // and the Dockerfile from memory
    let mut builder = Builder::new(Vec::new());
    builder
        .append_path_with_name(F1_EXT_INSTALL_PATH, "f1-ext-install")
        .unwrap();

    let mut header = Header::new_gnu();
    header.set_size(dockerfile.len().try_into().unwrap());
    header.set_mode(0o644);
    builder
        .append_data(&mut header, "Dockerfile", dockerfile.as_bytes())
        .unwrap();

    let body = builder.into_inner().unwrap();

    let options = BuildImageOptions {
        dockerfile: "Dockerfile",
        t: tag,

        buildargs: args.iter().cloned().collect(),

        nocache: true,

        ..Default::default()
    };

    // Block on the build future, panicking (i.e., failing the test) if the build returns
    // an error
    block_on(async {
        let mut stream = client.build_image(options, None, Some(body.into()));
        while let Some(result) = stream.next().await {
            let result = match result {
                Ok(result) => result,
                Err(err) => return Err(err.to_string()),
            };

            // Errors during `RUN` are propagated as BuildImageError, so we return the
            // value as an error message to the block_on() function.
            if let BuildImageResults::BuildImageError { ref error, .. } = result {
                return Err(error.clone());
            }

            println!("{:?}", result);
        }

        Ok(())
    })
    .unwrap();
}

/// Returns the Docker image tag for this test
pub fn tag_for_test(test_type: &str, package: &str, php_version: &str) -> String {
    format!(
        "f1-ext-install-test:{}-{}-{}",
        test_type, package, php_version
    )
}
