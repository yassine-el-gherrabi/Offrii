import XCTest
@testable import Offrii

@MainActor
final class AppRouterTests: XCTestCase {

    private var sut: AppRouter!

    override func setUp() {
        super.setUp()
        sut = AppRouter()
    }

    override func tearDown() {
        sut = nil
        super.tearDown()
    }

    func testDetermineInitialScreen_notOnboarded_notAuth_showsOnboarding() {
        sut.determineInitialScreen(isAuthenticated: false, hasSeenOnboarding: false)
        XCTAssertEqual(sut.currentScreen, .onboarding)
    }

    func testDetermineInitialScreen_notOnboarded_auth_showsOnboarding() {
        sut.determineInitialScreen(isAuthenticated: true, hasSeenOnboarding: false)
        XCTAssertEqual(sut.currentScreen, .onboarding)
    }

    func testDetermineInitialScreen_onboarded_notAuth_showsAuth() {
        sut.determineInitialScreen(isAuthenticated: false, hasSeenOnboarding: true)
        XCTAssertEqual(sut.currentScreen, .auth)
    }

    func testDetermineInitialScreen_onboarded_auth_showsMain() {
        sut.determineInitialScreen(isAuthenticated: true, hasSeenOnboarding: true)
        XCTAssertEqual(sut.currentScreen, .main)
    }

    func testDefaultScreen_isOnboarding() {
        XCTAssertEqual(sut.currentScreen, .onboarding)
    }

    func testCurrentScreen_canBeSetDirectly() {
        sut.currentScreen = .main
        XCTAssertEqual(sut.currentScreen, .main)

        sut.currentScreen = .auth
        XCTAssertEqual(sut.currentScreen, .auth)
    }
}
