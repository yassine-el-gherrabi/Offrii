import Foundation

@Observable
@MainActor
final class MyWishesViewModel {
    var wishes: [MyWish] = []
    var isLoading = false
    var error: String?

    // MARK: - Load

    func loadMyWishes() async {
        isLoading = true
        error = nil

        do {
            wishes = try await CommunityWishService.shared.listMyWishes()
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    // MARK: - Actions

    func closeWish(id: UUID) async {
        do {
            try await CommunityWishService.shared.closeWish(id: id)
            await loadMyWishes()
        } catch {
            self.error = error.localizedDescription
        }
    }
}
