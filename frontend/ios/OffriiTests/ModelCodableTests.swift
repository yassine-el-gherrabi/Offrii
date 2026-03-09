import XCTest
@testable import Offrii

final class ModelCodableTests: XCTestCase {

    private let decoder: JSONDecoder = {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        return decoder
    }()

    // MARK: - Item

    func testItemDecoding_snakeCase() throws {
        let json = """
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "iPhone 16",
            "description": "Latest iPhone",
            "url": "https://apple.com",
            "estimated_price": 999.99,
            "priority": 3,
            "category_id": "660e8400-e29b-41d4-a716-446655440000",
            "status": "active",
            "purchased_at": null,
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-01-15T10:30:00Z",
            "is_claimed": false
        }
        """.data(using: .utf8)!

        let item = try decoder.decode(Item.self, from: json)

        XCTAssertEqual(item.name, "iPhone 16")
        XCTAssertEqual(item.description, "Latest iPhone")
        XCTAssertEqual(item.url, "https://apple.com")
        XCTAssertEqual(item.priority, 3)
        XCTAssertEqual(item.status, "active")
        XCTAssertFalse(item.isClaimed)
        XCTAssertNil(item.purchasedAt)
        XCTAssertNotNil(item.categoryId)
    }

    func testItemPriorityLabel_low() {
        let item = makeItem(priority: 1)
        XCTAssertFalse(item.priorityLabel.isEmpty)
    }

    func testItemPriorityLabel_medium() {
        let item = makeItem(priority: 2)
        XCTAssertFalse(item.priorityLabel.isEmpty)
    }

    func testItemPriorityLabel_high() {
        let item = makeItem(priority: 3)
        XCTAssertFalse(item.priorityLabel.isEmpty)
    }

    func testItemIsActive() {
        let item = makeItem(status: "active")
        XCTAssertTrue(item.isActive)
        XCTAssertFalse(item.isPurchased)
    }

    func testItemIsPurchased() {
        let item = makeItem(status: "purchased")
        XCTAssertFalse(item.isActive)
        XCTAssertTrue(item.isPurchased)
    }

    // MARK: - User

    func testUserDecoding_snakeCase() throws {
        let json = """
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "email": "test@example.com",
            "username": "marie_a3f1",
            "display_name": "Marie",
            "reminder_freq": "weekly",
            "reminder_time": "09:00",
            "timezone": "Europe/Paris",
            "locale": "fr",
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-01-15T10:30:00Z"
        }
        """.data(using: .utf8)!

        let user = try decoder.decode(User.self, from: json)

        XCTAssertEqual(user.email, "test@example.com")
        XCTAssertEqual(user.username, "marie_a3f1")
        XCTAssertEqual(user.displayName, "Marie")
        XCTAssertEqual(user.reminderFreq, "weekly")
        XCTAssertEqual(user.timezone, "Europe/Paris")
    }

    func testUserDecoding_nullDisplayName() throws {
        let json = """
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "email": "test@example.com",
            "username": "test_b2c4",
            "display_name": null,
            "reminder_freq": "never",
            "reminder_time": "09:00",
            "timezone": "UTC",
            "locale": "en",
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-01-15T10:30:00Z"
        }
        """.data(using: .utf8)!

        let user = try decoder.decode(User.self, from: json)
        XCTAssertNil(user.displayName)
        XCTAssertEqual(user.username, "test_b2c4")
    }

    // MARK: - AuthTokens

    func testAuthTokensDecoding() throws {
        let json = """
        {
            "access_token": "eyJhbGciOiJIUzI1NiJ9.token",
            "refresh_token": "eyJhbGciOiJIUzI1NiJ9.refresh"
        }
        """.data(using: .utf8)!

        let tokens = try decoder.decode(AuthTokens.self, from: json)

        XCTAssertEqual(tokens.accessToken, "eyJhbGciOiJIUzI1NiJ9.token")
        XCTAssertEqual(tokens.refreshToken, "eyJhbGciOiJIUzI1NiJ9.refresh")
    }

    // MARK: - Category

    func testCategoryDecoding() throws {
        let json = """
        {
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Tech",
            "icon": "💻",
            "is_default": true,
            "position": 1,
            "created_at": "2026-01-15T10:30:00Z"
        }
        """.data(using: .utf8)!

        let category = try decoder.decode(Category.self, from: json)

        XCTAssertEqual(category.name, "Tech")
        XCTAssertEqual(category.icon, "💻")
        XCTAssertTrue(category.isDefault)
        XCTAssertEqual(category.position, 1)
    }

    // MARK: - ItemsListResponse

    func testItemsListResponseDecoding() throws {
        let json = """
        {
            "items": [],
            "total": 42,
            "page": 2,
            "per_page": 20
        }
        """.data(using: .utf8)!

        let response = try decoder.decode(ItemsListResponse.self, from: json)

        XCTAssertTrue(response.items.isEmpty)
        XCTAssertEqual(response.total, 42)
        XCTAssertEqual(response.page, 2)
        XCTAssertEqual(response.perPage, 20)
    }

    // MARK: - Helpers

    private func makeItem(priority: Int = 2, status: String = "active") -> Item {
        Item(
            id: UUID(),
            name: "Test Item",
            description: nil,
            url: nil,
            estimatedPrice: nil,
            priority: priority,
            categoryId: nil,
            status: status,
            purchasedAt: nil,
            createdAt: Date(),
            updatedAt: Date(),
            isClaimed: false
        )
    }
}
