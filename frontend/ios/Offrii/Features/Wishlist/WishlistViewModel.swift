import Foundation

@Observable
@MainActor
final class WishlistViewModel {
    var items: [Item] = []
    var categories: [CategoryResponse] = []
    var selectedCategoryIds: Set<UUID> = []
    var selectedStatus: String = "active"
    var sortField: String = "created_at"
    var sortOrder: String = "desc"
    var searchQuery: String = ""
    var isLoading = false
    var isLoadingMore = false
    var error: String?

    // Selection mode
    var isSelectMode = false
    var selectedItemIds: Set<UUID> = []
    var isDeleting = false

    // Shared items tracking (session-only)
    var sharedItemIds: Set<UUID> = []

    private var currentPage = 1
    private var totalItems = 0
    private let perPage = 20

    var hasMorePages: Bool {
        items.count < totalItems
    }

    // 0 = En cours, 1 = Réservé, 2 = Offert
    var filteredSegmentIndex: Int = 0

    /// Items filtered by local search query, selected categories, and segment
    var filteredItems: [Item] {
        var result = items

        // Segment filtering
        switch filteredSegmentIndex {
        case 1: // Réservé — active + claimed only
            result = result.filter { $0.isClaimed }
        case 2: // Offert — purchased (already filtered by API)
            break
        default: // En cours — all active items (reserved + not reserved)
            break
        }

        if !searchQuery.isEmpty {
            result = result.filter { $0.name.localizedCaseInsensitiveContains(searchQuery) }
        }

        if !selectedCategoryIds.isEmpty {
            result = result.filter { item in
                guard let catId = item.categoryId else { return false }
                return selectedCategoryIds.contains(catId)
            }
        }

        return result
    }

    // MARK: - Load

    func loadItems() async {
        isLoading = true
        error = nil
        currentPage = 1

        do {
            let response = try await ItemService.shared.listItems(
                status: selectedStatus,
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

    func quickAdd(name: String, price: Decimal? = nil, categoryId: UUID? = nil, priority: Int? = nil, imageUrl: String? = nil, links: [String]? = nil) async -> Bool {
        do {
            let item = try await ItemService.shared.createItem(
                name: name,
                estimatedPrice: price,
                priority: priority,
                categoryId: categoryId,
                imageUrl: imageUrl,
                links: links
            )
            if selectedStatus == "active" {
                items.insert(item, at: 0)
                totalItems += 1

                Task {
                    let updated = await ItemService.shared.refetchIfMissingOG(item)
                    if updated.ogImageUrl != nil, let idx = items.firstIndex(where: { $0.id == item.id }) {
                        items[idx] = updated
                    }
                }
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

    // MARK: - Batch Delete

    func toggleSelectMode() {
        isSelectMode.toggle()
        if !isSelectMode {
            selectedItemIds.removeAll()
        }
    }

    func toggleItemSelection(_ id: UUID) {
        if selectedItemIds.contains(id) {
            selectedItemIds.remove(id)
        } else {
            selectedItemIds.insert(id)
        }
    }

    func selectAllVisible() {
        selectedItemIds = Set(filteredItems.map(\.id))
    }

    func deselectAll() {
        selectedItemIds.removeAll()
    }

    func batchDelete() async {
        guard !selectedItemIds.isEmpty else { return }
        isDeleting = true

        do {
            try await ItemService.shared.batchDelete(ids: Array(selectedItemIds))
            items.removeAll { selectedItemIds.contains($0.id) }
            totalItems -= selectedItemIds.count
            selectedItemIds.removeAll()
            isSelectMode = false
        } catch {
            self.error = error.localizedDescription
        }
        isDeleting = false
    }

    // MARK: - Filters

    func toggleCategory(_ id: UUID) {
        if selectedCategoryIds.contains(id) {
            selectedCategoryIds.remove(id)
        } else {
            selectedCategoryIds.insert(id)
        }
        // No API reload needed — filtering is local
    }

    func clearCategoryFilters() {
        selectedCategoryIds.removeAll()
    }

    func selectOnlyCategory(_ id: UUID) {
        selectedCategoryIds = [id]
    }

    func changeSegment(_ index: Int) {
        filteredSegmentIndex = index
        // "Offert" (2) loads purchased, "En cours" (0) and "Réservé" (1) both load active
        selectedStatus = index == 2 ? "purchased" : "active"
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

    func category(for id: UUID?) -> CategoryResponse? {
        guard let id else { return nil }
        return categories.first { $0.id == id }
    }

    // MARK: - Sharing

    func markItemAsShared(_ id: UUID) {
        sharedItemIds.insert(id)
    }
}
