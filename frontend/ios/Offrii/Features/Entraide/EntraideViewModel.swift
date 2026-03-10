import Foundation

@Observable
@MainActor
final class EntraideViewModel {
    var wishes: [CommunityWish] = []
    var selectedCategory: WishCategory?
    var isLoading = false
    var isLoadingMore = false
    var error: String?

    private var currentPage = 1
    private var hasMore = true
    private let limit = 20

    var hasMorePages: Bool { hasMore }

    // MARK: - Load

    func loadWishes() async {
        isLoading = true
        error = nil
        currentPage = 1

        do {
            let response = try await CommunityWishService.shared.listWishes(
                category: selectedCategory?.rawValue,
                page: 1,
                limit: limit
            )
            wishes = response.data
            hasMore = response.pagination.hasMore
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    func loadMoreIfNeeded(currentWish: CommunityWish) async {
        guard hasMore,
              !isLoadingMore,
              let index = wishes.firstIndex(where: { $0.id == currentWish.id }),
              index >= wishes.count - 3 else { return }

        isLoadingMore = true
        currentPage += 1

        do {
            let response = try await CommunityWishService.shared.listWishes(
                category: selectedCategory?.rawValue,
                page: currentPage,
                limit: limit
            )
            wishes.append(contentsOf: response.data)
            hasMore = response.pagination.hasMore
        } catch {
            currentPage -= 1
        }
        isLoadingMore = false
    }

    // MARK: - Filters

    func selectCategory(_ category: WishCategory?) {
        selectedCategory = category
        Task { await loadWishes() }
    }

    // MARK: - Actions

    func offerWish(id: UUID) async -> Bool {
        do {
            try await CommunityWishService.shared.offerWish(id: id)
            await loadWishes()
            return true
        } catch {
            self.error = error.localizedDescription
            return false
        }
    }
}
