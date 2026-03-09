import Foundation

@Observable
@MainActor
final class CircleDetailViewModel {
    var detail: CircleDetailResponse?
    var items: [CircleItemResponse] = []
    var feed: [CircleEventResponse] = []
    var selectedTab: DetailTab = .items
    var isLoading = false
    var error: String?

    enum DetailTab: String, CaseIterable {
        case items
        case activity
    }

    var currentUserId: UUID?

    /// Items grouped by member: current user's items first, then others.
    var itemsByMember: [(member: CircleMember, items: [CircleItemResponse])] {
        guard let detail else { return [] }
        let grouped = Dictionary(grouping: items) { $0.sharedBy }
        var result: [(member: CircleMember, items: [CircleItemResponse])] = []

        // Current user first
        if let userId = currentUserId,
           let member = detail.members.first(where: { $0.userId == userId }),
           let memberItems = grouped[userId], !memberItems.isEmpty {
            result.append((member: member, items: memberItems))
        }

        // Other members
        for member in detail.members {
            if member.userId == currentUserId { continue }
            if let memberItems = grouped[member.userId], !memberItems.isEmpty {
                result.append((member: member, items: memberItems))
            }
        }

        return result
    }

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
}
