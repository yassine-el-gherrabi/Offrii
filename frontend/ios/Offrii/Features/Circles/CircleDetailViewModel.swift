import Foundation

@Observable
@MainActor
final class CircleDetailViewModel {
    var detail: CircleDetailResponse?
    var items: [CircleItemResponse] = []
    var feed: [CircleEventResponse] = []
    var selectedTab: DetailTab = .items
    var selectedMemberFilter: UUID?
    var isLoading = false
    var error: String?
    var currentUserId: UUID?

    enum DetailTab: String, CaseIterable {
        case items
        case myItems
        case members
        case activity
    }

    // MARK: - Computed Properties

    var filteredItems: [CircleItemResponse] {
        guard let memberId = selectedMemberFilter else { return items }
        return items.filter { $0.sharedBy == memberId }
    }

    var myItems: [CircleItemResponse] {
        guard let userId = currentUserId else { return [] }
        return items.filter { $0.sharedBy == userId }
    }

    var theirItems: [CircleItemResponse] {
        guard let userId = currentUserId else { return items }
        return items.filter { $0.sharedBy != userId }
    }

    /// Available tabs depend on whether the circle is direct (1:1) or a group.
    var availableTabs: [DetailTab] {
        if detail?.isDirect == true {
            return [.items, .myItems, .activity]
        }
        return [.items, .myItems, .members, .activity]
    }

    /// The friend member in a 1:1 circle (not the current user).
    var friendMember: CircleMember? {
        guard detail?.isDirect == true, let userId = currentUserId else { return nil }
        return detail?.members.first { $0.userId != userId }
    }

    // MARK: - Data Loading

    func loadDetail(circleId: UUID) async {
        isLoading = true
        error = nil
        do {
            detail = try await CircleService.shared.getCircle(id: circleId)
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    func loadItems(circleId: UUID) async {
        do {
            items = try await CircleService.shared.listItems(circleId: circleId)
        } catch {
            self.error = error.localizedDescription
        }
    }

    func loadFeed(circleId: UUID) async {
        do {
            let response = try await CircleService.shared.getFeed(circleId: circleId)
            feed = response.events
        } catch {
            self.error = error.localizedDescription
        }
    }

    // MARK: - Actions

    func claimItem(itemId: UUID) async {
        do {
            try await CircleService.shared.claimItem(itemId: itemId)
        } catch {
            self.error = error.localizedDescription
        }
    }

    func unclaimItem(itemId: UUID) async {
        do {
            try await CircleService.shared.unclaimItem(itemId: itemId)
        } catch {
            self.error = error.localizedDescription
        }
    }

    func removeMember(circleId: UUID, userId: UUID) async {
        do {
            try await CircleService.shared.removeMember(circleId: circleId, userId: userId)
            detail?.members.removeAll { $0.userId == userId }
        } catch {
            self.error = error.localizedDescription
        }
    }

    func leaveCircle(circleId: UUID) async -> Bool {
        guard let userId = currentUserId else { return false }
        do {
            try await CircleService.shared.removeMember(circleId: circleId, userId: userId)
            return true
        } catch {
            self.error = error.localizedDescription
            return false
        }
    }
}
