import Foundation

final class FriendService: Sendable {
    static let shared = FriendService()
    private let client = APIClient.shared
    private init() {}

    func searchUsers(query: String) async throws -> [UserSearchResult] {
        try await client.request(.searchUsers(query: query))
    }

    func sendRequest(username: String) async throws -> FriendRequestResponse {
        try await client.request(.sendFriendRequest(SendFriendRequestBody(username: username)))
    }

    func listPendingRequests() async throws -> [FriendRequestResponse] {
        let response: PaginatedResponse<FriendRequestResponse> = try await client.request(.listPendingFriendRequests)
        return response.data
    }

    func listSentRequests() async throws -> [SentFriendRequestResponse] {
        let response: PaginatedResponse<SentFriendRequestResponse> = try await client.request(.listSentFriendRequests)
        return response.data
    }

    func cancelRequest(id: UUID) async throws {
        try await client.requestVoid(.cancelFriendRequest(id: id))
    }

    func acceptRequest(id: UUID) async throws -> FriendResponse {
        try await client.request(.acceptFriendRequest(id: id))
    }

    func declineRequest(id: UUID) async throws {
        try await client.requestVoid(.declineFriendRequest(id: id))
    }

    func listFriends() async throws -> [FriendResponse] {
        let response: PaginatedResponse<FriendResponse> = try await client.request(.listFriends)
        return response.data
    }

    func removeFriend(userId: UUID) async throws {
        try await client.requestVoid(.removeFriend(userId: userId))
    }
}
