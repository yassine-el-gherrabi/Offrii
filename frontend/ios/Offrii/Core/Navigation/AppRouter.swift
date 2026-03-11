import SwiftUI

// MARK: - App Screen

enum AppScreen {
    case splash
    case welcome       // first-launch onboarding
    case auth
    case postAuthSetup // after register only
    case main
}

// MARK: - App Router

@Observable
final class AppRouter {
    var currentScreen: AppScreen = .splash

    /// Whether the user has already seen the onboarding welcome screens.
    var hasSeenWelcome: Bool {
        get { UserDefaults.standard.bool(forKey: "hasSeenWelcome") }
        set { UserDefaults.standard.set(newValue, forKey: "hasSeenWelcome") }
    }

    /// Whether this is a returning user (has launched before and seen welcome).
    var isReturningUser: Bool { hasSeenWelcome }

    /// Called from SplashView after auth check completes.
    func resolvePostSplash(isAuthenticated: Bool) {
        if isAuthenticated {
            currentScreen = .main
        } else if !hasSeenWelcome {
            currentScreen = .welcome
        } else {
            currentScreen = .auth
        }
    }

    /// Marks onboarding as complete and navigates to auth.
    func completeOnboarding() {
        hasSeenWelcome = true
        currentScreen = .auth
    }

    /// Called after post-auth setup finishes (or is skipped).
    func completePostAuthSetup() {
        currentScreen = .main
    }
}
