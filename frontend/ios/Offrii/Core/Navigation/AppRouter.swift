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

    /// Whether the auth screen should start on register (true) or login (false).
    var preferRegister = true

    /// Marks onboarding as complete and navigates to auth (register).
    func completeOnboarding() {
        hasSeenWelcome = true
        preferRegister = true
        currentScreen = .auth
    }

    /// Marks onboarding as complete and navigates to auth (login).
    func completeOnboardingToLogin() {
        hasSeenWelcome = true
        preferRegister = false
        currentScreen = .auth
    }

    /// Called after post-auth setup finishes (or is skipped).
    func completePostAuthSetup() {
        currentScreen = .main
    }

    /// Pending circle invite token from deep link (offrii://join/{token})
    var pendingInviteToken: String?

    /// Pending circle navigation from push notification tap or notification center
    var pendingCircleId: UUID?

    /// Triggers switching to the friends filter in Circles tab
    var showFriends = false

    func handleURL(_ url: URL) {
        // offrii://join/{token}
        guard url.scheme == "offrii", url.host == "join",
              let token = url.pathComponents.last, !token.isEmpty, token != "/"
        else { return }
        pendingInviteToken = token
    }

    func navigateToCircle(_ circleId: UUID) {
        pendingCircleId = circleId
    }
}
