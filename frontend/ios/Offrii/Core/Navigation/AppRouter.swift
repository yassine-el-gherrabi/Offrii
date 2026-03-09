import SwiftUI

// MARK: - App Screen

enum AppScreen {
    case onboarding
    case auth
    case main
}

// MARK: - App Router

@Observable
final class AppRouter {
    var currentScreen: AppScreen = .onboarding

    /// Determines the initial screen based on authentication and onboarding state.
    ///
    /// - Parameters:
    ///   - isAuthenticated: Whether the user has a valid session.
    ///   - hasSeenOnboarding: Whether onboarding has been completed previously.
    func determineInitialScreen(isAuthenticated: Bool, hasSeenOnboarding: Bool) {
        if !hasSeenOnboarding {
            currentScreen = .onboarding
        } else if isAuthenticated {
            currentScreen = .main
        } else {
            currentScreen = .auth
        }
    }
}
