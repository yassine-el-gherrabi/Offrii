import Foundation

@Observable
@MainActor
final class WishlistViewModel {
    var items: [Item] = []
    var categories: [CategoryResponse] = []
    var selectedCategoryId: UUID?
    var selectedStatus: String = "active"
    var sortField: String = "created_at"
    var sortOrder: String = "desc"
    var isLoading = false
    var isLoadingMore = false
    var error: String?

    private var currentPage = 1
    private var totalItems = 0
    private let perPage = 20

    var hasMorePages: Bool {
        items.count < totalItems
    }

    var filteredSegmentIndex: Int {
        get { selectedStatus == "active" ? 0 : 1 }
        set { selectedStatus = newValue == 0 ? "active" : "purchased" }
    }

    // MARK: - Load

    func loadItems() async {
        isLoading = true
        error = nil
        currentPage = 1

        do {
            let response = try await ItemService.shared.listItems(
                status: selectedStatus,
                categoryId: selectedCategoryId,
                sort: sortField,
                order: sortOrder,
                page: 1,
                perPage: perPage
            )
            items = response.items
            totalItems = response.total
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    func loadMoreIfNeeded(currentItem: Item) async {
        guard hasMorePages,
              !isLoadingMore,
              let index = items.firstIndex(where: { $0.id == currentItem.id }),
              index >= items.count - 3 else { return }

        isLoadingMore = true
        currentPage += 1

        do {
            let response = try await ItemService.shared.listItems(
                status: selectedStatus,
                categoryId: selectedCategoryId,
                sort: sortField,
                order: sortOrder,
                page: currentPage,
                perPage: perPage
            )
            items.append(contentsOf: response.items)
            totalItems = response.total
        } catch {
            currentPage -= 1
        }
        isLoadingMore = false
    }

    func loadCategories() async {
        do {
            categories = try await CategoryService.shared.listCategories()
        } catch {
            // Keep empty
        }
    }

    // MARK: - CRUD

    func quickAdd(name: String) async -> Bool {
        do {
            let item = try await ItemService.shared.createItem(name: name)
            if selectedStatus == "active" {
                items.insert(item, at: 0)
                totalItems += 1
            }
            return true
        } catch {
            self.error = error.localizedDescription
            return false
        }
    }

    func deleteItem(_ item: Item) async {
        do {
            try await ItemService.shared.deleteItem(id: item.id)
            items.removeAll { $0.id == item.id }
            totalItems -= 1
        } catch {
            self.error = error.localizedDescription
        }
    }

    func markPurchased(_ item: Item) async {
        do {
            _ = try await ItemService.shared.updateItem(id: item.id, status: "purchased")
            items.removeAll { $0.id == item.id }
            totalItems -= 1
        } catch {
            self.error = error.localizedDescription
        }
    }

    // MARK: - Filters

    func selectCategory(_ id: UUID?) {
        selectedCategoryId = id
        Task { await loadItems() }
    }

    func changeSegment(_ index: Int) {
        filteredSegmentIndex = index
        Task { await loadItems() }
    }

    func changeSort(_ field: String) {
        if sortField == field {
            sortOrder = sortOrder == "asc" ? "desc" : "asc"
        } else {
            sortField = field
            sortOrder = field == "priority" ? "desc" : "asc"
        }
        Task { await loadItems() }
    }

    func categoryName(for id: UUID?) -> String? {
        guard let id else { return nil }
        return categories.first { $0.id == id }?.name
    }
}
