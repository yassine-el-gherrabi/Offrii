import XCTest
@testable import Offrii

@MainActor
final class ProfileViewModelTests: XCTestCase {

    private var sut: ProfileViewModel!

    override func setUp() {
        super.setUp()
        sut = ProfileViewModel()
    }

    override func tearDown() {
        sut = nil
        super.tearDown()
    }

    // MARK: - Initials

    func testInitials_twoNames() {
        sut.displayName = "Marie Dupont"
        XCTAssertEqual(sut.initials, "MD")
    }

    func testInitials_threeNames_usesFirstTwo() {
        sut.displayName = "Jean Pierre Martin"
        XCTAssertEqual(sut.initials, "JP")
    }

    func testInitials_singleName() {
        sut.displayName = "Marie"
        XCTAssertEqual(sut.initials, "MA")
    }

    func testInitials_emptyDisplayName_usesEmail() {
        sut.displayName = ""
        sut.email = "test@example.com"
        XCTAssertEqual(sut.initials, "TE")
    }

    func testInitials_uppercased() {
        sut.displayName = "jean dupont"
        XCTAssertEqual(sut.initials, "JD")
    }

    // MARK: - Reminder Frequency Label

    func testReminderFreqLabel_daily() {
        sut.reminderFreq = "daily"
        XCTAssertFalse(sut.reminderFreqLabel.isEmpty)
    }

    func testReminderFreqLabel_weekly() {
        sut.reminderFreq = "weekly"
        XCTAssertFalse(sut.reminderFreqLabel.isEmpty)
    }

    func testReminderFreqLabel_monthly() {
        sut.reminderFreq = "monthly"
        XCTAssertFalse(sut.reminderFreqLabel.isEmpty)
    }

    func testReminderFreqLabel_never() {
        sut.reminderFreq = "never"
        XCTAssertFalse(sut.reminderFreqLabel.isEmpty)
    }

    func testReminderFreqLabel_unknown_defaultsToNever() {
        sut.reminderFreq = "unknown_value"
        let neverLabel = {
            self.sut.reminderFreq = "never"
            return self.sut.reminderFreqLabel
        }()
        sut.reminderFreq = "unknown_value"
        XCTAssertEqual(sut.reminderFreqLabel, neverLabel)
    }

    // MARK: - Default State

    func testDefaultState() {
        XCTAssertEqual(sut.displayName, "")
        XCTAssertEqual(sut.username, "")
        XCTAssertEqual(sut.email, "")
        XCTAssertEqual(sut.reminderFreq, "never")
        XCTAssertEqual(sut.reminderTime, "09:00")
        XCTAssertFalse(sut.isLoggingOut)
    }
}
