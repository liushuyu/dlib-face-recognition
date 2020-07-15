#[cfg(feature = "embed-any")]
fn download_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("files")
}

#[cfg(feature = "embed-any")]
fn download_and_unzip(client: &reqwest::Client, url: &str) {
    use bzip2::read::*;

    let url: reqwest::Url = url.parse().unwrap();

    let filename = url
        .path_segments()
        .unwrap()
        .last()
        .unwrap()
        .replace(".bz2", "");

    let path = download_path().join(&filename);

    if path.exists() {
        println!("Already got '{}'", path.display());
        return;
    }

    println!("Downloading '{}'...", url);

    let response = client.get(url).send().unwrap();
    let mut decoded = BzDecoder::new(response);
    let mut file = std::fs::File::create(&path).unwrap();
    std::io::copy(&mut decoded, &mut file).unwrap();
}

fn build_dlib() {
    use cc::Build;
    println!("{:?}", std::env::current_dir());
    let files: &[&str] = &[
        "dlib/dlib/base64/base64_kernel_1.cpp",
        "dlib/dlib/bigint/bigint_kernel_1.cpp",
        "dlib/dlib/bigint/bigint_kernel_2.cpp",
        "dlib/dlib/bit_stream/bit_stream_kernel_1.cpp",
        "dlib/dlib/entropy_decoder/entropy_decoder_kernel_1.cpp",
        "dlib/dlib/entropy_decoder/entropy_decoder_kernel_2.cpp",
        "dlib/dlib/entropy_encoder/entropy_encoder_kernel_1.cpp",
        "dlib/dlib/entropy_encoder/entropy_encoder_kernel_2.cpp",
        "dlib/dlib/md5/md5_kernel_1.cpp",
        "dlib/dlib/tokenizer/tokenizer_kernel_1.cpp",
        "dlib/dlib/unicode/unicode.cpp",
        "dlib/dlib/test_for_odr_violations.cpp",
    ];
    println!("cargo:rerun-if-changed=./dlib/");
    Build::new()
        .files(files)
        .define("DLIB_ISO_CPP_ONLY", None)
        .define("DLIB_DISABLE_ASSERTS", None)
        .define(
            "DLIB_CHECK_FOR_VERSION_MISMATCH",
            "DLIB_VERSION_MISMATCH_CHECK__EXPECTED_VERSION_19_20_99",
        )
        .compile("libdlib.a");
}

fn main() {
    let mut config = cpp_build::Config::new();

    if cfg!(feature = "bundled-dlib") {
        build_dlib();
        println!("cargo:rustc-link-lib=dlib");
        config.include("./dlib/").build("src/lib.rs");
    } else {
        println!("cargo:rustc-link-lib=dlib");
        println!("cargo:rustc-link-lib=lapack");
        println!("cargo:rustc-link-lib=cblas");
        config.build("src/lib.rs");
    }

    #[cfg(feature = "embed-any")]
    {
        if !download_path().exists() {
            std::fs::create_dir(download_path()).unwrap();
        }

        // Download the data files
        // I'm not sure if doing this in the build script is such a good idea, seeing as it happens silently,
        // but I dont think adding the files to the repo is good either

        // Create a client for maintaining connections
        let client = reqwest::ClientBuilder::new()
            // Turn off gzip decryption
            // See: https://github.com/seanmonstar/reqwest/issues/328
            .gzip(false)
            .build()
            .unwrap();

        #[cfg(feature = "embed-fd-nn")]
        download_and_unzip(
            &client,
            "http://dlib.net/files/mmod_human_face_detector.dat.bz2",
        );
        #[cfg(feature = "embed-fe-nn")]
        download_and_unzip(
            &client,
            "http://dlib.net/files/dlib_face_recognition_resnet_model_v1.dat.bz2",
        );
        #[cfg(feature = "embed-lp")]
        download_and_unzip(
            &client,
            "http://dlib.net/files/shape_predictor_5_face_landmarks.dat.bz2",
        );
    }
}
