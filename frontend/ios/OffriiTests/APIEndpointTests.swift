import XCTest
@testable import Offrii

final class APIEndpointTests: XCTestCase {

    // MARK: - Auth Paths

    func testPath_register() {
        let endpoint = APIEndpoint.register(RegisterBody(email: "a@b.com", password: "pass", displayName: nil))
        XCTAssertEqual(endpoint.path, "/auth/register")
    }

    func testPath_login() {
        let endpoint = APIEndpoint.login(LoginBody(email: "a@b.com", password: "pass"))
        XCTAssertEqual(endpoint.path, "/auth/login")
    }

    func testPath_refresh() {
        let endpoint = APIEndpoint.refresh(RefreshBody(refreshToken: "tok"))
        XCTAssertEqual(endpoint.path, "/auth/refresh")
    }

    func testPath_logout() {
        XCTAssertEqual(APIEndpoint.logout.path, "/auth/logout")
    }

    func testPath_forgotPassword() {
        let endpoint = APIEndpoint.forgotPassword(ForgotPasswordBody(email: "a@b.com"))
        XCTAssertEqual(endpoint.path, "/auth/forgot-password")
    }

    func testPath_resetPassword() {
        let endpoint = APIEndpoint.resetPassword(ResetPasswordBody(email: "a@b.com", code: "123456", newPassword: "new"))
        XCTAssertEqual(endpoint.path, "/auth/reset-password")
    }

    // MARK: - Item Paths

    func testPath_listItems() {
        let endpoint = APIEndpoint.listItems(ListItemsQuery())
        XCTAssertEqual(endpoint.path, "/items")
    }

    func testPath_createItem() {
        let endpoint = APIEndpoint.createItem(CreateItemBody(name: "Test", description: nil, url: nil, estimatedPrice: nil, priority: nil, categoryId: nil))
        XCTAssertEqual(endpoint.path, "/items")
    }

    func testPath_getItem() {
        let id = UUID()
        XCTAssertEqual(APIEndpoint.getItem(id: id).path, "/items/\(id)")
    }

    func testPath_deleteItem() {
        let id = UUID()
        XCTAssertEqual(APIEndpoint.deleteItem(id: id).path, "/items/\(id)")
    }

    func testPath_claimItem() {
        let id = UUID()
        XCTAssertEqual(APIEndpoint.claimItem(id: id).path, "/items/\(id)/claim")
    }

    // MARK: - User Paths

    func testPath_getProfile() {
        XCTAssertEqual(APIEndpoint.getProfile.path, "/users/me")
    }

    func testPath_deleteAccount() {
        XCTAssertEqual(APIEndpoint.deleteAccount.path, "/users/me")
    }

    func testPath_exportData() {
        XCTAssertEqual(APIEndpoint.exportData.path, "/users/me/export")
    }

    // MARK: - Share Link Paths

    func testPath_listShareLinks() {
        XCTAssertEqual(APIEndpoint.listShareLinks.path, "/share-links")
    }

    func testPath_getSharedView() {
        XCTAssertEqual(APIEndpoint.getSharedView(token: "abc123").path, "/shared/abc123")
    }

    func testPath_claimViaShare() {
        let itemId = UUID()
        XCTAssertEqual(
            APIEndpoint.claimViaShare(token: "abc", itemId: itemId).path,
            "/shared/abc/items/\(itemId)/claim"
        )
    }

    // MARK: - HTTP Methods

    func testMethod_register_isPOST() {
        let endpoint = APIEndpoint.register(RegisterBody(email: "", password: "", displayName: nil))
        XCTAssertEqual(endpoint.method, .POST)
    }

    func testMethod_login_isPOST() {
        let endpoint = APIEndpoint.login(LoginBody(email: "", password: ""))
        XCTAssertEqual(endpoint.method, .POST)
    }

    func testMethod_listItems_isGET() {
        XCTAssertEqual(APIEndpoint.listItems(ListItemsQuery()).method, .GET)
    }

    func testMethod_deleteItem_isDELETE() {
        XCTAssertEqual(APIEndpoint.deleteItem(id: UUID()).method, .DELETE)
    }

    func testMethod_updateItem_isPUT() {
        let body = UpdateItemBody(name: nil, description: nil, url: nil, estimatedPrice: nil, priority: nil, categoryId: nil, status: nil)
        XCTAssertEqual(APIEndpoint.updateItem(id: UUID(), body: body).method, .PUT)
    }

    func testMethod_updateProfile_isPATCH() {
        let body = UpdateProfileBody(displayName: nil, username: nil, reminderFreq: nil, reminderTime: nil, timezone: nil, locale: nil)
        XCTAssertEqual(APIEndpoint.updateProfile(body).method, .PATCH)
    }

    func testMethod_getProfile_isGET() {
        XCTAssertEqual(APIEndpoint.getProfile.method, .GET)
    }

    // MARK: - Requires Auth

    func testRequiresAuth_register_false() {
        let endpoint = APIEndpoint.register(RegisterBody(email: "", password: "", displayName: nil))
        XCTAssertFalse(endpoint.requiresAuth)
    }

    func testRequiresAuth_login_false() {
        let endpoint = APIEndpoint.login(LoginBody(email: "", password: ""))
        XCTAssertFalse(endpoint.requiresAuth)
    }

    func testRequiresAuth_refresh_false() {
        let endpoint = APIEndpoint.refresh(RefreshBody(refreshToken: ""))
        XCTAssertFalse(endpoint.requiresAuth)
    }

    func testRequiresAuth_forgotPassword_false() {
        let endpoint = APIEndpoint.forgotPassword(ForgotPasswordBody(email: ""))
        XCTAssertFalse(endpoint.requiresAuth)
    }

    func testRequiresAuth_resetPassword_false() {
        let endpoint = APIEndpoint.resetPassword(ResetPasswordBody(email: "", code: "", newPassword: ""))
        XCTAssertFalse(endpoint.requiresAuth)
    }

    func testRequiresAuth_getSharedView_false() {
        XCTAssertFalse(APIEndpoint.getSharedView(token: "abc").requiresAuth)
    }

    func testRequiresAuth_logout_true() {
        XCTAssertTrue(APIEndpoint.logout.requiresAuth)
    }

    func testRequiresAuth_getProfile_true() {
        XCTAssertTrue(APIEndpoint.getProfile.requiresAuth)
    }

    func testRequiresAuth_listItems_true() {
        XCTAssertTrue(APIEndpoint.listItems(ListItemsQuery()).requiresAuth)
    }

    func testRequiresAuth_deleteAccount_true() {
        XCTAssertTrue(APIEndpoint.deleteAccount.requiresAuth)
    }

    // MARK: - Query Items

    func testQueryItems_listItems_allParams() {
        let categoryId = UUID()
        let query = ListItemsQuery(status: "active", categoryId: categoryId, sort: "priority", order: "desc", page: 2, perPage: 10)
        let endpoint = APIEndpoint.listItems(query)
        let items = endpoint.queryItems!

        XCTAssertTrue(items.contains(.init(name: "status", value: "active")))
        XCTAssertTrue(items.contains(.init(name: "category_id", value: categoryId.uuidString)))
        XCTAssertTrue(items.contains(.init(name: "sort", value: "priority")))
        XCTAssertTrue(items.contains(.init(name: "order", value: "desc")))
        XCTAssertTrue(items.contains(.init(name: "page", value: "2")))
        XCTAssertTrue(items.contains(.init(name: "per_page", value: "10")))
    }

    func testQueryItems_listItems_noParams_nil() {
        let query = ListItemsQuery()
        let endpoint = APIEndpoint.listItems(query)
        XCTAssertNil(endpoint.queryItems)
    }

    func testQueryItems_nonListEndpoint_nil() {
        XCTAssertNil(APIEndpoint.getProfile.queryItems)
        XCTAssertNil(APIEndpoint.logout.queryItems)
    }

    // MARK: - Body

    func testBody_register_notNil() {
        let endpoint = APIEndpoint.register(RegisterBody(email: "a@b.com", password: "pass", displayName: "Test"))
        XCTAssertNotNil(endpoint.body)
    }

    func testBody_login_notNil() {
        let endpoint = APIEndpoint.login(LoginBody(email: "a@b.com", password: "pass"))
        XCTAssertNotNil(endpoint.body)
    }

    func testBody_logout_nil() {
        XCTAssertNil(APIEndpoint.logout.body)
    }

    func testBody_getProfile_nil() {
        XCTAssertNil(APIEndpoint.getProfile.body)
    }

    func testBody_deleteItem_nil() {
        XCTAssertNil(APIEndpoint.deleteItem(id: UUID()).body)
    }
}
