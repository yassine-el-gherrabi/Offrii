import Foundation

@Observable
@MainActor
final class WishDetailViewModel {
    var wish: WishDetail?
    var isLoading = false
    var isActioning = false
    var error: String?
    var actionSuccess: String?

    // MARK: - Load

    func loadWish(id: UUID) async {
        isLoading = true
        error = nil

        do {
            wish = try await CommunityWishService.shared.getWish(id: id)
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    // MARK: - Actions

    func offer(id: UUID) async -> Bool {
        isActioning = true
        do {
            try await CommunityWishService.shared.offerWish(id: id)
            await loadWish(id: id)
            actionSuccess = NSLocalizedString("entraide.action.offerSuccess", comment: "")
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
            await loadWish(id: id)
            actionSuccess = NSLocalizedString("entraide.action.withdrawSuccess", comment: "")
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
            await loadWish(id: id)
            actionSuccess = NSLocalizedString("entraide.action.confirmSuccess", comment: "")
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
