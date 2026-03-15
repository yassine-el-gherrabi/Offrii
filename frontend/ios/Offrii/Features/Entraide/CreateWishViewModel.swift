import Foundation
import UIKit

@Observable
@MainActor
final class CreateWishViewModel {
    var title = ""
    var description = ""
    var selectedCategory: WishCategory?
    var isAnonymous = false
    var selectedImage: UIImage?
    var links: [String] = [""]
    var isSubmitting = false
    var error: String?

    // MARK: - Validation

    var isTitleValid: Bool { !title.trimmingCharacters(in: .whitespaces).isEmpty && title.count <= 255 }
    var isCategoryValid: Bool { selectedCategory != nil }
    var isFormValid: Bool { isTitleValid && isCategoryValid }
    var descriptionCount: Int { description.count }
    var isDescriptionOverLimit: Bool { description.count > 2000 }

    // MARK: - Links Management

    func addLink() {
        guard links.count < 10 else { return }
        links.append("")
    }

    func removeLink(at index: Int) {
        guard links.count > 1 else {
            links = [""]
            return
        }
        links.remove(at: index)
    }

    // MARK: - Submit

    func submit() async -> Bool {
        guard isFormValid, !isDescriptionOverLimit else { return false }

        isSubmitting = true
        error = nil

        // Upload image if selected
        var imageUrl: String?
        if let image = selectedImage,
           let data = image.compressForUpload() {
            do {
                imageUrl = try await ItemService.shared.uploadImage(data)
            } catch {
                self.error = error.localizedDescription
                isSubmitting = false
                return false
            }
        }

        let trimmedLinks = links
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }

        do {
            _ = try await CommunityWishService.shared.createWish(
                title: title.trimmingCharacters(in: .whitespaces),
                description: description.isEmpty ? nil : description,
                category: selectedCategory!,
                isAnonymous: isAnonymous,
                imageUrl: imageUrl,
                links: trimmedLinks.isEmpty ? nil : trimmedLinks
            )
            isSubmitting = false
            return true
        } catch {
            self.error = error.localizedDescription
            isSubmitting = false
            return false
        }
    }
}
