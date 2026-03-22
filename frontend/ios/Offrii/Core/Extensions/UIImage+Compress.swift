import UIKit

extension UIImage {
    /// Compress image for upload. No resize — the server handles sizing based on type.
    /// - Parameter maxSizeMB: Maximum file size in MB (default 5.0)
    /// - Returns: JPEG Data under maxSizeMB, or nil if compression fails
    func compressForUpload(maxSizeMB: Double = 5.0) -> Data? {
        let maxBytes = Int(maxSizeMB * 1024 * 1024)
        let qualities: [CGFloat] = [0.85, 0.7, 0.5, 0.3]
        for quality in qualities {
            if let data = jpegData(compressionQuality: quality),
               data.count <= maxBytes {
                return data
            }
        }
        return jpegData(compressionQuality: 0.1)
    }
}
