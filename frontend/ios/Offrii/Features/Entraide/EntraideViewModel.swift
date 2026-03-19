import Foundation

@Observable
@MainActor
final class EntraideViewModel {
    // Discover
    var wishes: [CommunityWish] = []
    var selectedCategory: WishCategory?
    var isLoading = false
    var isLoadingMore = false
    var error: String?

    // Sort
    var sortField = "created_at"
    var sortOrder = "desc"

    // My Offers (separate data source)
    var myOfferWishes: [CommunityWish] = []
    var isLoadingOffers = false

    private var currentPage = 1
    private var hasMore = true
    private let limit = 20

    var filteredWishes: [CommunityWish] {
        let sorted: [CommunityWish]
        switch sortField {
        case "title":
            sorted = wishes.sorted { lhs, rhs in
                sortOrder == "asc"
                    ? lhs.title.localizedCaseInsensitiveCompare(rhs.title) == .orderedAscending
                    : lhs.title.localizedCaseInsensitiveCompare(rhs.title) == .orderedDescending
            }
        default: // created_at
            sorted = wishes.sorted { lhs, rhs in
                sortOrder == "asc" ? lhs.createdAt < rhs.createdAt : lhs.createdAt > rhs.createdAt
            }
        }
        return sorted
    }

    // MARK: - Load Discover

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
        let nextPage = currentPage + 1

        do {
            let response = try await CommunityWishService.shared.listWishes(
                category: selectedCategory?.rawValue,
                page: nextPage,
                limit: limit
            )
            wishes.append(contentsOf: response.data)
            hasMore = response.pagination.hasMore
            currentPage = nextPage
        } catch {}
        isLoadingMore = false
    }

    // MARK: - Load My Offers (independent from Discover)

    func loadMyOffers() async {
        isLoadingOffers = true
        do {
            myOfferWishes = try await CommunityWishService.shared.listMyOffers()
        } catch {}
        isLoadingOffers = false
    }

    // MARK: - Category Filter

    func selectCategory(_ category: WishCategory?) async {
        selectedCategory = category
        await loadWishes()
    }

    // MARK: - Actions

    func offerWish(id: UUID) async -> Bool {
        do {
            try await CommunityWishService.shared.offerWish(id: id)
            OffriiHaptics.success()
            await loadWishes()
            return true
        } catch {
            self.error = error.localizedDescription
            return false
        }
    }
}
