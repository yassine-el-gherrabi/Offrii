import XCTest
@testable import Offrii

final class ItemModelTests: XCTestCase {

    private let decoder: JSONDecoder = {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        return decoder
    }()

    // MARK: - Helper

    private func makeItemJSON(
        claimedVia: String? = nil,
        claimedName: String? = nil,
        isClaimed: Bool = false,
        imageUrl: String? = nil,
        ogImageUrl: String? = nil,
        sharedCircles: [[String: Any]]? = nil,
        isPrivate: Bool = false
    ) -> Data {
        var dict: [String: Any] = [
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Test Item",
            "priority": 2,
            "status": "active",
            "created_at": "2026-01-15T10:30:00Z",
            "updated_at": "2026-01-15T10:30:00Z",
            "is_claimed": isClaimed,
            "is_private": isPrivate,
        ]
        if let val = claimedVia { dict["claimed_via"] = val }
        if let val = claimedName { dict["claimed_name"] = val }
        if let val = imageUrl { dict["image_url"] = val }
        if let val = ogImageUrl { dict["og_image_url"] = val }
        if let val = sharedCircles { dict["shared_circles"] = val }
        // swiftlint:disable:next force_try
        return try! JSONSerialization.data(withJSONObject: dict) // Safe in tests
    }

    // MARK: - Claimed Via

    func testDecodeItemWithClaimedViaWeb() throws {
        let json = makeItemJSON(claimedVia: "web", claimedName: "Marie", isClaimed: true)
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertEqual(item.claimedVia, "web")
        XCTAssertEqual(item.claimedName, "Marie")
        XCTAssertTrue(item.isClaimed)
    }

    func testDecodeItemWithClaimedViaApp() throws {
        let json = makeItemJSON(claimedVia: "app", isClaimed: true)
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertEqual(item.claimedVia, "app")
        XCTAssertNil(item.claimedName)
    }

    func testDecodeItemWithoutClaimedVia() throws {
        let json = makeItemJSON()
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertNil(item.claimedVia)
        XCTAssertNil(item.claimedName)
        XCTAssertFalse(item.isClaimed)
    }

    func testIsWebClaim() throws {
        let json = makeItemJSON(claimedVia: "web", isClaimed: true)
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertTrue(item.isWebClaim)
        XCTAssertFalse(item.isAppClaim)
    }

    func testIsAppClaim() throws {
        let json = makeItemJSON(claimedVia: "app", isClaimed: true)
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertFalse(item.isWebClaim)
        XCTAssertTrue(item.isAppClaim)
    }

    func testNotClaimedIsNeitherWebNorApp() throws {
        let json = makeItemJSON()
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertFalse(item.isWebClaim)
        XCTAssertFalse(item.isAppClaim)
    }

    // MARK: - Display Image URL Priority

    func testDisplayImageUrl_imageUrlFirst() throws {
        let json = makeItemJSON(imageUrl: "https://example.com/upload.jpg", ogImageUrl: "https://example.com/og.jpg")
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertEqual(item.displayImageUrl?.absoluteString, "https://example.com/upload.jpg")
    }

    func testDisplayImageUrl_ogFallback() throws {
        let json = makeItemJSON(ogImageUrl: "https://example.com/og.jpg")
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertEqual(item.displayImageUrl?.absoluteString, "https://example.com/og.jpg")
    }

    func testDisplayImageUrl_nilWhenNoImages() throws {
        let json = makeItemJSON()
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertNil(item.displayImageUrl)
    }

    // MARK: - Shared Circles

    func testDecodeItemWithSharedCircles() throws {
        let circles: [[String: Any]] = [
            ["id": "660e8400-e29b-41d4-a716-446655440000", "name": "Famille", "is_direct": false],
            ["id": "770e8400-e29b-41d4-a716-446655440000", "name": "Nicolas", "is_direct": true],
        ]
        let json = makeItemJSON(sharedCircles: circles)
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertEqual(item.sharedCircles.count, 2)
        XCTAssertEqual(item.sharedCircles[0].name, "Famille")
        XCTAssertEqual(item.sharedCircles[0].isDirect, false)
        XCTAssertEqual(item.sharedCircles[1].name, "Nicolas")
        XCTAssertEqual(item.sharedCircles[1].isDirect, true)
    }

    func testDecodeItemWithEmptySharedCircles() throws {
        let json = makeItemJSON(sharedCircles: [])
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertTrue(item.sharedCircles.isEmpty)
    }

    func testDecodeItemWithoutSharedCirclesField() throws {
        // When the field is missing entirely, should default to empty
        let json = makeItemJSON()
        let item = try decoder.decode(Item.self, from: json)

        XCTAssertTrue(item.sharedCircles.isEmpty)
    }
}

// MARK: - SharedCircleInfo Tests

final class SharedCircleInfoTests: XCTestCase {

    private let decoder = JSONDecoder()

    func testDecodeGroupCircle() throws {
        let json = """
        {"id": "550e8400-e29b-41d4-a716-446655440000", "name": "Famille", "is_direct": false}
        """.data(using: .utf8)!

        let info = try decoder.decode(SharedCircleInfo.self, from: json)

        XCTAssertEqual(info.name, "Famille")
        XCTAssertEqual(info.isDirect, false)
    }

    func testDecodeDirectCircle() throws {
        let json = """
        {"id": "550e8400-e29b-41d4-a716-446655440000", "name": "Nicolas", "is_direct": true}
        """.data(using: .utf8)!

        let info = try decoder.decode(SharedCircleInfo.self, from: json)

        XCTAssertEqual(info.name, "Nicolas")
        XCTAssertEqual(info.isDirect, true)
    }

    func testDecodeWithoutIsDirect() throws {
        // is_direct is optional — should default to nil
        let json = """
        {"id": "550e8400-e29b-41d4-a716-446655440000", "name": "Test"}
        """.data(using: .utf8)!

        let info = try decoder.decode(SharedCircleInfo.self, from: json)

        XCTAssertNil(info.isDirect)
    }

    func testInitialUppercased() throws {
        let json = """
        {"id": "550e8400-e29b-41d4-a716-446655440000", "name": "famille", "is_direct": false}
        """.data(using: .utf8)!

        let info = try decoder.decode(SharedCircleInfo.self, from: json)

        XCTAssertEqual(info.initial, "F")
    }

    func testInitialEmptyName() throws {
        let json = """
        {"id": "550e8400-e29b-41d4-a716-446655440000", "name": "", "is_direct": false}
        """.data(using: .utf8)!

        let info = try decoder.decode(SharedCircleInfo.self, from: json)

        XCTAssertEqual(info.initial, "")
    }
}
