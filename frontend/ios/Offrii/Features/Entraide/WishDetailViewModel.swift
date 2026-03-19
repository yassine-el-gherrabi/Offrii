import Foundation

@Observable
@MainActor
final class WishDetailViewModel {
    var wish: WishDetail?
    var isLoading = false
    var isActioning = false
    var error: String?
    var actionSuccess: String?

    func loadWish(id: UUID) async {
        isLoading = true
        do {
            wish = try await CommunityWishService.shared.getWish(id: id)
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    func offer(id: UUID) async -> Bool {
        isActioning = true
        do {
            try await CommunityWishService.shared.offerWish(id: id)
            actionSuccess = NSLocalizedString("entraide.action.offerSuccess", comment: "")
            OffriiHaptics.success()
            await loadWish(id: id)
            isActioning = false
            return true
        } catch {
            self.error = error.localizedDescription
            isActioning = false
            return false
        }
    }

    func withdrawOffer(id: UUID) async -> Bool {
        isActioning = true
        do {
            try await CommunityWishService.shared.withdrawOffer(id: id)
            actionSuccess = NSLocalizedString("entraide.action.withdrawSuccess", comment: "")
            OffriiHaptics.success()
            await loadWish(id: id)
            isActioning = false
            return true
        } catch {
            self.error = error.localizedDescription
            isActioning = false
            return false
        }
    }

    func confirm(id: UUID) async -> Bool {
        isActioning = true
        do {
            try await CommunityWishService.shared.confirmWish(id: id)
            actionSuccess = NSLocalizedString("entraide.action.confirmSuccess", comment: "")
            OffriiHaptics.success()
            await loadWish(id: id)
            isActioning = false
            return true
        } catch {
            self.error = error.localizedDescription
            isActioning = false
            return false
        }
    }

    func rejectOffer(id: UUID) async -> Bool {
        isActioning = true
        do {
            try await CommunityWishService.shared.rejectOffer(id: id)
            OffriiHaptics.success()
            await loadWish(id: id)
            isActioning = false
            return true
        } catch {
            self.error = error.localizedDescription
            isActioning = false
            return false
        }
    }

    func closeWish(id: UUID) async -> Bool {
        isActioning = true
        do {
            try await CommunityWishService.shared.closeWish(id: id)
            OffriiHaptics.success()
            await loadWish(id: id)
            isActioning = false
            return true
        } catch {
            self.error = error.localizedDescription
            isActioning = false
            return false
        }
    }
}
