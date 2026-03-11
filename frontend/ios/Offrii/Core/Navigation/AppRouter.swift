import SwiftUI

// MARK: - App Screen

enum AppScreen {
    case auth
    case main
}

// MARK: - App Router

@Observable
final class AppRouter {
    var currentScreen: AppScreen = .auth

    func determineInitialScreen(isAuthenticated: Bool) {
        if isAuthenticated {
            currentScreen = .main
        } else {
            currentScreen = .auth
        }
    }
}
