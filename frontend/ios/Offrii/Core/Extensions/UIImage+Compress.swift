import UIKit

extension UIImage {
    /// Compress image for upload: resize if too large, then JPEG compress.
    /// - Parameters:
    ///   - maxDimension: Maximum width or height in pixels (default 1600)
    ///   - maxSizeMB: Maximum file size in MB (default 4.0)
    /// - Returns: JPEG Data under maxSizeMB, or nil if compression fails
    func compressForUpload(maxDimension: CGFloat = 1600, maxSizeMB: Double = 4.0) -> Data? {
        let maxBytes = Int(maxSizeMB * 1024 * 1024)

        // Step 1: Resize if needed
        var image = self
        let longestSide = max(size.width, size.height)
        if longestSide > maxDimension {
            let scale = maxDimension / longestSide
            let newSize = CGSize(width: size.width * scale, height: size.height * scale)
            let renderer = UIGraphicsImageRenderer(size: newSize)
            image = renderer.image { _ in
                self.draw(in: CGRect(origin: .zero, size: newSize))
            }
        }

        // Step 2: Compress with decreasing quality until under maxBytes
        let qualities: [CGFloat] = [0.8, 0.6, 0.4, 0.2]
        for quality in qualities {
            if let data = image.jpegData(compressionQuality: quality),
               data.count <= maxBytes {
                return data
            }
        }

        // Last resort: lowest quality
        return image.jpegData(compressionQuality: 0.1)
    }
}
