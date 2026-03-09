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

        async let f: Result<[FriendResponse], Error> = Result { try await FriendService.shared.listFriends() }
        async let p: Result<[FriendRequestResponse], Error> = Result { try await FriendService.shared.listPendingRequests() }
        async let s: Result<[SentFriendRequestResponse], Error> = Result { try await FriendService.shared.listSentRequests() }

        let (fResult, pResult, sResult) = await (f, p, s)

        if case .success(let val) = fResult { friends = val }
        if case .success(let val) = pResult { pendingRequests = val }
        if case .success(let val) = sResult { sentRequests = val }

        // Show error only if all three failed
        var errors: [Error] = []
        if case .failure(let e) = fResult { errors.append(e) }
        if case .failure(let e) = pResult { errors.append(e) }
        if case .failure(let e) = sResult { errors.append(e) }
        if errors.count == 3 {
            self.error = errors.first?.localizedDescription
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
