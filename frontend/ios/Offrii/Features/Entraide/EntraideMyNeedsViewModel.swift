import Foundation

@Observable
@MainActor
final class EntraideMyNeedsViewModel {
    var wishes: [MyWish] = []
    var isLoading = false
    var error: String?

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

    func closeWish(id: UUID) async {
        do {
            try await CommunityWishService.shared.closeWish(id: id)
            OffriiHaptics.success()
            await loadMyWishes()
        } catch {
            self.error = error.localizedDescription
        }
    }

    func confirmWish(id: UUID) async {
        do {
            try await CommunityWishService.shared.confirmWish(id: id)
            OffriiHaptics.success()
            await loadMyWishes()
        } catch {
            self.error = error.localizedDescription
        }
    }

    func deleteWish(id: UUID) async {
        do {
            try await CommunityWishService.shared.deleteWish(id: id)
            OffriiHaptics.success()
            await loadMyWishes()
        } catch {
            self.error = error.localizedDescription
        }
    }

    func reopenWish(id: UUID) async {
        do {
            try await CommunityWishService.shared.reopenWish(id: id)
            OffriiHaptics.success()
            await loadMyWishes()
        } catch {
            self.error = error.localizedDescription
        }
    }
}
