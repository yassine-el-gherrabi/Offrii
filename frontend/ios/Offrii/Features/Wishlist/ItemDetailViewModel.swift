import Foundation

@Observable
@MainActor
final class ItemDetailViewModel {
    var item: Item?
    var isLoading = false
    var isUpdating = false
    var error: String?
    var categoryName: String?
    var categoryIcon: String?

    var style: CategoryStyle {
        CategoryStyle(icon: categoryIcon)
    }

    func updateItem(_ newItem: Item) {
        item = newItem
        Task { await loadCategoryInfo() }
    }

    func loadItem(id: UUID) async {
        isLoading = true
        do {
            item = try await ItemService.shared.getItem(id: id)
            await loadCategoryInfo()
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    func loadCircleItem(circleId: UUID, itemId: UUID) async {
        isLoading = true
        do {
            let circleItem = try await CircleService.shared.getItem(
                circleId: circleId,
                itemId: itemId
            )
            item = Item.fromCircleItem(circleItem)
            // Use category_icon from the backend directly (avoids per-user category mismatch)
            categoryIcon = circleItem.categoryIcon
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    private func loadCategoryInfo() async {
        if let categoryId = item?.categoryId {
            let categories = (try? await CategoryService.shared.listCategories()) ?? []
            let cat = categories.first { $0.id == categoryId }
            categoryName = cat?.name
            categoryIcon = cat?.icon
        }
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

    func unarchive() async -> Bool {
        guard let item else { return false }
        isUpdating = true
        do {
            self.item = try await ItemService.shared.updateItem(id: item.id, status: "active")
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
