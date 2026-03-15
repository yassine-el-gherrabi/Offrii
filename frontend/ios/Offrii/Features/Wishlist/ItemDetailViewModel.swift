import Foundation

@Observable
@MainActor
final class ItemDetailViewModel {
    var item: Item?
    var isLoading = false
    var isUpdating = false
    var error: String?
    var categoryName: String?

    func loadItem(id: UUID) async {
        isLoading = true
        do {
            item = try await ItemService.shared.getItem(id: id)
            if let categoryId = item?.categoryId {
                let categories = try await CategoryService.shared.listCategories()
                categoryName = categories.first { $0.id == categoryId }?.name
            }
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    func markPurchased() async -> Bool {
        guard let item else { return false }
        isUpdating = true
        do {
            self.item = try await ItemService.shared.updateItem(id: item.id, status: "purchased")
            isUpdating = false
            return true
        } catch {
            self.error = error.localizedDescription
            isUpdating = false
            return false
        }
    }

    func deleteItem() async -> Bool {
        guard let item else { return false }
        do {
            try await ItemService.shared.deleteItem(id: item.id)
            return true
        } catch {
            self.error = error.localizedDescription
            return false
        }
    }

    func unshareFromCircle(circleId: UUID) async {
        guard let item else { return }
        do {
            try await CircleService.shared.unshareItem(circleId: circleId, itemId: item.id)
            self.item = try await ItemService.shared.getItem(id: item.id)
        } catch {
            self.error = error.localizedDescription
        }
    }

    /// Owner removes a web claim from their item.
    func ownerUnclaimWeb() async {
        guard let item else { return }
        isUpdating = true
        do {
            try await ItemService.shared.ownerUnclaimWeb(id: item.id)
            // Reload the item to reflect the change
            self.item = try await ItemService.shared.getItem(id: item.id)
        } catch {
            self.error = error.localizedDescription
        }
        isUpdating = false
    }
}
