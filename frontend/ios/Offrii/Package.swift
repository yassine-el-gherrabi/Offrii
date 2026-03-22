// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "OffriiDependencies",
    platforms: [.iOS(.v17)],
    dependencies: [
        .package(url: "https://github.com/kishikawakatsumi/KeychainAccess.git", from: "4.2.2"),
        .package(url: "https://github.com/kean/Nuke.git", from: "12.0.0"),
        .package(url: "https://github.com/google/GoogleSignIn-iOS.git", from: "8.0.0"),
    ],
    targets: []
)
