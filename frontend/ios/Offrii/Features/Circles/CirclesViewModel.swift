import Foundation

@Observable
@MainActor
final class CirclesViewModel {

    // MARK: - Circles State

    var circles: [OffriiCircle] = []
    var isLoadingCircles = false

    // MARK: - Friends State

    var friends: [FriendResponse] = []
    var pendingRequests: [FriendRequestResponse] = []
    var sentRequests: [SentFriendRequestResponse] = []
    var isLoadingFriends = false
    var friendSearchQuery = ""
    var circleSearchQuery = ""

    // MARK: - Shared State

    var error: String?

    // MARK: - Computed

    var pendingCount: Int {
        pendingRequests.count
    }

    var filteredCircles: [OffriiCircle] {
        let trimmed = circleSearchQuery.trimmingCharacters(in: .whitespaces).lowercased()
        guard !trimmed.isEmpty else { return circles }
        return circles.filter { circle in
            circle.name?.lowercased().contains(trimmed) ?? false
        }
    }

    var filteredFriends: [FriendResponse] {
        let trimmed = friendSearchQuery.trimmingCharacters(in: .whitespaces).lowercased()
        guard !trimmed.isEmpty else { return friends }
        return friends.filter { friend in
            let displayMatch = friend.displayName?.lowercased().contains(trimmed) ?? false
            let usernameMatch = friend.username.lowercased().contains(trimmed)
            return displayMatch || usernameMatch
        }
    }

    /// Find the direct (1:1) circle with a specific friend by matching user ID.
    func directCircle(for friend: FriendResponse) -> OffriiCircle? {
        circles.first { $0.isDirect && $0.memberIds.contains(friend.userId) }
    }

    // MARK: - Load All (Parallel)

    func loadAll() async {
        async let circlesTask: () = loadCircles()
        async let friendsTask: () = loadFriends()
        async let pendingTask: () = loadPendingRequests()
        async let sentTask: () = loadSentRequests()
        _ = await (circlesTask, friendsTask, pendingTask, sentTask)
    }

    // MARK: - Circles

    func loadCircles() async {
        isLoadingCircles = true
        error = nil
        do {
            circles = try await CircleService.shared.listCircles()
        } catch {
            self.error = error.localizedDescription
        }
        isLoadingCircles = false
    }

    func deleteCircle(_ circle: OffriiCircle) async {
        do {
            try await CircleService.shared.deleteCircle(id: circle.id)
            circles.removeAll { $0.id == circle.id }
        } catch {
            self.error = error.localizedDescription
        }
    }

    // MARK: - Friends

    func loadFriends() async {
        isLoadingFriends = true
        do {
            friends = try await FriendService.shared.listFriends()
        } catch {
            if self.error == nil {
                self.error = error.localizedDescription
            }
        }
        isLoadingFriends = false
    }

    func loadPendingRequests() async {
        do {
            pendingRequests = try await FriendService.shared.listPendingRequests()
        } catch { /* Best-effort refresh */ }
    }

    func loadSentRequests() async {
        do {
            sentRequests = try await FriendService.shared.listSentRequests()
        } catch { /* Best-effort refresh */ }
    }

    // MARK: - Friend Actions

    func acceptRequest(_ request: FriendRequestResponse) async {
        do {
            _ = try await FriendService.shared.acceptRequest(id: request.id)
            pendingRequests.removeAll { $0.id == request.id }
            async let friendsTask: () = loadFriends()
            async let circlesTask: () = loadCircles()
            _ = await (friendsTask, circlesTask)
        } catch {
            self.error = error.localizedDescription
        }
    }

    func declineRequest(_ request: FriendRequestResponse) async {
        do {
            try await FriendService.shared.declineRequest(id: request.id)
            pendingRequests.removeAll { $0.id == request.id }
        } catch {
            self.error = error.localizedDescription
        }
    }

    func cancelRequest(_ request: SentFriendRequestResponse) async {
        do {
            try await FriendService.shared.cancelRequest(id: request.id)
            sentRequests.removeAll { $0.id == request.id }
        } catch {
            self.error = error.localizedDescription
        }
    }

    func removeFriend(_ friend: FriendResponse) async {
        do {
            try await FriendService.shared.removeFriend(userId: friend.userId)
            friends.removeAll { $0.userId == friend.userId }
        } catch {
            self.error = error.localizedDescription
        }
    }
}
