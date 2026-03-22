import XCTest
@testable import Offrii

// swiftlint:disable file_length

/// Tests the JSON decoding contract between the Rust backend and Swift models.
/// If the backend changes a field name or type, these tests catch it.
final class ModelDecodingTests: XCTestCase {

    private let decoder: JSONDecoder = {
        let d = JSONDecoder()
        d.dateDecodingStrategy = .iso8601
        return d
    }()

    // MARK: - User

    func testUserDecoding() throws {
        let json = Data("""
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "email": "emma@demo.com",
            "username": "emma",
            "display_name": "Emma",
            "avatar_url": "https://cdn.offrii.com/demo/avatar-emma.jpg",
            "email_verified": true,
            "username_customized": true,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-01-15T10:30:00Z"
        }
        """.utf8)

        let user = try decoder.decode(User.self, from: json)
        XCTAssertEqual(user.email, "emma@demo.com")
        XCTAssertEqual(user.displayName, "Emma")
        XCTAssertEqual(user.avatarUrl, "https://cdn.offrii.com/demo/avatar-emma.jpg")
        XCTAssertEqual(user.emailVerified, true)
    }

    func testUserDecoding_nullOptionals() throws {
        let json = Data("""
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "email": "test@test.com",
            "username": "test",
            "display_name": null,
            "avatar_url": null,
            "email_verified": false,
            "username_customized": false,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-01-15T10:30:00Z"
        }
        """.utf8)

        let user = try decoder.decode(User.self, from: json)
        XCTAssertNil(user.displayName)
        XCTAssertNil(user.avatarUrl)
    }

    // MARK: - AuthTokens

    func testAuthTokensDecoding() throws {
        let json = Data("""
        {
            "access_token": "eyJ.access",
            "refresh_token": "eyJ.refresh"
        }
        """.utf8)

        let tokens = try decoder.decode(AuthTokens.self, from: json)
        XCTAssertEqual(tokens.accessToken, "eyJ.access")
        XCTAssertEqual(tokens.refreshToken, "eyJ.refresh")
    }

    // MARK: - Category

    func testCategoryDecoding() throws {
        let json = Data("""
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Tech",
            "icon": "desktopcomputer",
            "is_default": true,
            "position": 1,
            "created_at": "2026-01-15T10:30:00Z"
        }
        """.utf8)

        let cat = try decoder.decode(CategoryResponse.self, from: json)
        XCTAssertEqual(cat.name, "Tech")
        XCTAssertEqual(cat.icon, "desktopcomputer")
        XCTAssertTrue(cat.isDefault)
    }

    // MARK: - Circle

    func testCircleDecoding_group() throws {
        let json = Data("""
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Famille",
            "is_direct": false,
            "owner_id": "660e8400-e29b-41d4-a716-446655440000",
            "image_url": null,
            "member_count": 4,
            "unreserved_item_count": 3,
            "last_activity": "item_shared",
            "last_activity_at": "2026-01-15T10:30:00Z",
            "member_names": ["Emma", "Marie"],
            "member_ids": ["660e8400-e29b-41d4-a716-446655440000"],
            "member_avatars": [null],
            "created_at": "2026-01-15T10:30:00Z"
        }
        """.utf8)

        let circle = try decoder.decode(OffriiCircle.self, from: json)
        XCTAssertEqual(circle.name, "Famille")
        XCTAssertFalse(circle.isDirect)
        XCTAssertEqual(circle.memberCount, 4)
    }

    func testCircleDecoding_direct() throws {
        let json = Data("""
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": null,
            "is_direct": true,
            "owner_id": "660e8400-e29b-41d4-a716-446655440000",
            "member_count": 2,
            "member_names": ["Marie"],
            "member_ids": ["770e8400-e29b-41d4-a716-446655440000"],
            "member_avatars": [null],
            "created_at": "2026-01-15T10:30:00Z"
        }
        """.utf8)

        let circle = try decoder.decode(OffriiCircle.self, from: json)
        XCTAssertNil(circle.name)
        XCTAssertTrue(circle.isDirect)
        XCTAssertEqual(circle.memberCount, 2)
    }

    // MARK: - Friend

    func testFriendDecoding() throws {
        let json = Data("""
        {
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "username": "marie",
            "display_name": "Marie L.",
            "since": "2026-01-10T08:00:00Z",
            "shared_item_count": 5
        }
        """.utf8)

        let friend = try decoder.decode(FriendResponse.self, from: json)
        XCTAssertEqual(friend.username, "marie")
        XCTAssertEqual(friend.displayName, "Marie L.")
        XCTAssertEqual(friend.sharedItemCount, 5)
    }

    func testFriendRequestDecoding() throws {
        let json = Data("""
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "from_user_id": "660e8400-e29b-41d4-a716-446655440000",
            "from_username": "lucas",
            "from_display_name": "Lucas",
            "status": "pending",
            "created_at": "2026-01-15T10:30:00Z"
        }
        """.utf8)

        let req = try decoder.decode(FriendRequestResponse.self, from: json)
        XCTAssertEqual(req.fromUsername, "lucas")
        XCTAssertEqual(req.status, "pending")
    }

    // MARK: - Notification

    func testNotificationDecoding() throws {
        let json = Data("""
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "item_claimed",
            "title": "Reservation",
            "body": "Someone reserved your item",
            "read": false,
            "circle_id": null,
            "item_id": "660e8400-e29b-41d4-a716-446655440000",
            "actor_id": "770e8400-e29b-41d4-a716-446655440000",
            "actor_name": "Marie",
            "created_at": "2026-01-15T10:30:00Z"
        }
        """.utf8)

        let notif = try decoder.decode(AppNotification.self, from: json)
        XCTAssertEqual(notif.type, "item_claimed")
        XCTAssertFalse(notif.read)
        XCTAssertEqual(notif.actorName, "Marie")
        XCTAssertEqual(notif.icon, "gift.fill")
    }

    func testNotificationIcon_friendRequest() {
        let notif = makeNotification(type: "friend_request")
        XCTAssertEqual(notif.icon, "person.badge.plus")
    }

    func testNotificationIcon_circleActivity() {
        let notif = makeNotification(type: "circle_activity")
        XCTAssertEqual(notif.icon, "person.2.fill")
    }

    func testNotificationIcon_unknown() {
        let notif = makeNotification(type: "some_future_type")
        XCTAssertEqual(notif.icon, "bell.fill")
    }

    // MARK: - CommunityWish

    func testCommunityWishDecoding() throws {
        let json = Data("""
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "display_name": "Sarah",
            "title": "Warm clothes for winter",
            "description": "Size M/L",
            "category": "clothing",
            "status": "open",
            "is_mine": false,
            "is_matched_by_me": false,
            "has_reported": false,
            "image_url": null,
            "links": null,
            "fulfilled_at": null,
            "created_at": "2026-01-15T10:30:00Z"
        }
        """.utf8)

        let wish = try decoder.decode(CommunityWish.self, from: json)
        XCTAssertEqual(wish.title, "Warm clothes for winter")
        XCTAssertEqual(wish.displayName, "Sarah")
        XCTAssertFalse(wish.isMine)
        XCTAssertEqual(wish.hasReported, false)
    }

    // MARK: - CircleMember

    func testCircleMemberDecoding() throws {
        let json = Data("""
        {
            "user_id": "550e8400-e29b-41d4-a716-446655440000",
            "username": "sophie",
            "display_name": "Sophie",
            "avatar_url": null,
            "role": "member",
            "joined_at": "2026-01-15T10:30:00Z"
        }
        """.utf8)

        let member = try decoder.decode(CircleMember.self, from: json)
        XCTAssertEqual(member.username, "sophie")
        XCTAssertEqual(member.role, "member")
    }

    // MARK: - CircleDetail

    func testCircleDetailDecoding() throws {
        let json = Data("""
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Famille",
            "is_direct": false,
            "owner_id": "660e8400-e29b-41d4-a716-446655440000",
            "image_url": null,
            "members": [],
            "created_at": "2026-01-15T10:30:00Z"
        }
        """.utf8)

        let detail = try decoder.decode(CircleDetailResponse.self, from: json)
        XCTAssertEqual(detail.name, "Famille")
        XCTAssertFalse(detail.isDirect)
        XCTAssertTrue(detail.members.isEmpty)
    }

    // MARK: - Helpers

    private func makeNotification(type: String) -> AppNotification {
        let json = Data("""
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "type": "\(type)",
            "title": "Test",
            "body": "Test body",
            "read": false,
            "circle_id": null,
            "item_id": null,
            "actor_id": null,
            "actor_name": null,
            "created_at": "2026-01-15T10:30:00Z"
        }
        """.utf8)
        // swiftlint:disable:next force_try
        return try! decoder.decode(AppNotification.self, from: json)
    }
}

// swiftlint:enable file_length
