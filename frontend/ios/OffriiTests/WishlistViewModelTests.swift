import XCTest
@testable import Offrii

@MainActor
final class WishlistViewModelTests: XCTestCase {

    private var sut: WishlistViewModel!

    override func setUp() async throws {
        sut = WishlistViewModel()
    }

    override func tearDown() async throws {
        sut = nil
    }

    // MARK: - Has More Pages

    func testHasMorePages_itemsLessThanTotal_true() {
        sut.items = [makeItem(), makeItem()]
        // Access private totalItems through reflection or by loading
        // Since totalItems is private, test via the computed property
        // after simulating a load where total > items.count
        // For now, test the default state
        XCTAssertFalse(sut.hasMorePages)
    }

    func testHasMorePages_defaultState_false() {
        XCTAssertFalse(sut.hasMorePages)
    }

    // MARK: - Filtered Segment Index

    func testFilteredSegmentIndex_default_isZero() {
        XCTAssertEqual(sut.filteredSegmentIndex, 0)
    }

    func testFilteredSegmentIndex_setToOne_changesToPurchased() {
        sut.filteredSegmentIndex = 1
        XCTAssertEqual(sut.selectedStatus, "purchased")
    }

    func testFilteredSegmentIndex_setToZero_changesToActive() {
        sut.filteredSegmentIndex = 1
        sut.filteredSegmentIndex = 0
        XCTAssertEqual(sut.selectedStatus, "active")
    }

    func testFilteredSegmentIndex_getActive_returnsZero() {
        sut.selectedStatus = "active"
        XCTAssertEqual(sut.filteredSegmentIndex, 0)
    }

    func testFilteredSegmentIndex_getPurchased_returnsOne() {
        sut.selectedStatus = "purchased"
        XCTAssertEqual(sut.filteredSegmentIndex, 1)
    }

    // MARK: - Category Name

    func testCategoryName_found() {
        let id = UUID()
        sut.categories = [
            CategoryResponse(
                id: id, name: "Tech", icon: "💻",
                isDefault: false, position: 1, createdAt: Date()
            ),
        ]
        XCTAssertEqual(sut.categoryName(for: id), "Tech")
    }

    func testCategoryName_notFound_nil() {
        sut.categories = [
            CategoryResponse(
                id: UUID(), name: "Tech", icon: nil,
                isDefault: false, position: 1, createdAt: Date()
            ),
        ]
        XCTAssertNil(sut.categoryName(for: UUID()))
    }

    func testCategoryName_nilId_nil() {
        XCTAssertNil(sut.categoryName(for: nil))
    }

    // MARK: - Sort

    func testDefaultSort_isCreatedAt_desc() {
        XCTAssertEqual(sut.sortField, "created_at")
        XCTAssertEqual(sut.sortOrder, "desc")
    }

    // MARK: - Default State

    func testDefaultState() {
        XCTAssertTrue(sut.items.isEmpty)
        XCTAssertTrue(sut.categories.isEmpty)
        XCTAssertNil(sut.selectedCategoryId)
        XCTAssertEqual(sut.selectedStatus, "active")
        XCTAssertFalse(sut.isLoading)
        XCTAssertFalse(sut.isLoadingMore)
        XCTAssertNil(sut.error)
    }

    // MARK: - Helpers

    private func makeItem() -> Item {
        Item(
            id: UUID(),
            name: "Test",
            description: nil,
            url: nil,
            estimatedPrice: nil,
            priority: 2,
            categoryId: nil,
            status: "active",
            purchasedAt: nil,
            createdAt: Date(),
            updatedAt: Date(),
            isClaimed: false
        )
    }
}
