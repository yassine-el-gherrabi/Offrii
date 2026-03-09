import XCTest
@testable import Offrii

@MainActor
final class AuthViewModelTests: XCTestCase {

    private var sut: AuthViewModel!

    override func setUp() async throws {
        sut = AuthViewModel()
    }

    override func tearDown() async throws {
        sut = nil
    }

    // MARK: - Email Validation

    func testValidateEmail_empty_returnsError() {
        sut.email = ""
        XCTAssertFalse(sut.validateEmail())
        XCTAssertNotNil(sut.emailError)
    }

    func testValidateEmail_whitespaceOnly_returnsError() {
        sut.email = "   "
        XCTAssertFalse(sut.validateEmail())
        XCTAssertNotNil(sut.emailError)
    }

    func testValidateEmail_invalidFormat_returnsError() {
        sut.email = "not-an-email"
        XCTAssertFalse(sut.validateEmail())
        XCTAssertNotNil(sut.emailError)
    }

    func testValidateEmail_missingDomain_returnsError() {
        sut.email = "user@"
        XCTAssertFalse(sut.validateEmail())
        XCTAssertNotNil(sut.emailError)
    }

    func testValidateEmail_missingTLD_returnsError() {
        sut.email = "user@domain"
        XCTAssertFalse(sut.validateEmail())
        XCTAssertNotNil(sut.emailError)
    }

    func testValidateEmail_valid_noError() {
        sut.email = "user@example.com"
        XCTAssertTrue(sut.validateEmail())
        XCTAssertNil(sut.emailError)
    }

    func testValidateEmail_validWithPlus_noError() {
        sut.email = "user+tag@example.com"
        XCTAssertTrue(sut.validateEmail())
        XCTAssertNil(sut.emailError)
    }

    func testValidateEmail_overrideParameter() {
        sut.email = "invalid"
        XCTAssertTrue(sut.validateEmail("valid@example.com"))
    }

    // MARK: - Password Validation

    func testValidatePassword_empty_returnsError() {
        sut.password = ""
        XCTAssertFalse(sut.validatePassword())
        XCTAssertNotNil(sut.passwordError)
    }

    func testValidatePassword_tooShort_returnsError() {
        sut.password = "1234567"
        XCTAssertFalse(sut.validatePassword())
        XCTAssertNotNil(sut.passwordError)
    }

    func testValidatePassword_exactly8_valid() {
        sut.password = "12345678"
        XCTAssertTrue(sut.validatePassword())
        XCTAssertNil(sut.passwordError)
    }

    func testValidatePassword_long_valid() {
        sut.password = "a-very-long-password-123"
        XCTAssertTrue(sut.validatePassword())
        XCTAssertNil(sut.passwordError)
    }

    // MARK: - New Password Validation

    func testValidateNewPassword_empty_returnsError() {
        sut.newPassword = ""
        XCTAssertFalse(sut.validateNewPassword())
        XCTAssertNotNil(sut.newPasswordError)
    }

    func testValidateNewPassword_tooShort_returnsError() {
        sut.newPassword = "short"
        XCTAssertFalse(sut.validateNewPassword())
        XCTAssertNotNil(sut.newPasswordError)
    }

    func testValidateNewPassword_valid() {
        sut.newPassword = "newpassword123"
        XCTAssertTrue(sut.validateNewPassword())
        XCTAssertNil(sut.newPasswordError)
    }

    // MARK: - State

    func testIsLoading_idle_false() {
        sut.state = .idle
        XCTAssertFalse(sut.isLoading)
    }

    func testIsLoading_loading_true() {
        sut.state = .loading
        XCTAssertTrue(sut.isLoading)
    }

    func testIsLoading_error_false() {
        sut.state = .error("Something failed")
        XCTAssertFalse(sut.isLoading)
    }

    // MARK: - Forgot Password Flow

    func testForgotStep_initialState_isEmail() {
        XCTAssertEqual(sut.forgotStep, .email)
    }

    func testResetForgotState_clearsAll() {
        sut.forgotStep = .code
        sut.resetEmail = "test@example.com"
        sut.resetCode = "123456"
        sut.newPassword = "password"
        sut.state = .error("error")

        sut.resetForgotState()

        XCTAssertEqual(sut.forgotStep, .email)
        XCTAssertEqual(sut.resetEmail, "")
        XCTAssertEqual(sut.resetCode, "")
        XCTAssertEqual(sut.newPassword, "")
        XCTAssertEqual(sut.state, .idle)
    }
}
