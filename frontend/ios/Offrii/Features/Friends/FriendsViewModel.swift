import Foundation

@Observable
@MainActor
final class FriendsViewModel {
    var friends: [FriendResponse] = []
    var pendingRequests: [FriendRequestResponse] = []
    var sentRequests: [SentFriendRequestResponse] = []
    var isLoading = false
    var error: String?

    func loadAll() async {
        isLoading = true
        error = nil
        async let friendsTask = FriendService.shared.listFriends()
        async let requestsTask = FriendService.shared.listPendingRequests()
        async let sentTask = FriendService.shared.listSentRequests()
        do {
            friends = try await friendsTask
            pendingRequests = try await requestsTask
            sentRequests = try await sentTask
        } catch {
            self.error = error.localizedDescription
        }
        isLoading = false
    }

    func acceptRequest(_ request: FriendRequestResponse) async {
        do {
            _ = try await FriendService.shared.acceptRequest(id: request.id)
            pendingRequests.removeAll { $0.id == request.id }
            await loadFriends()
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

    private func loadFriends() async {
        do {
            friends = try await FriendService.shared.listFriends()
        } catch {
            // Already handled
        }
    }
}
