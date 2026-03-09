import SwiftUI

@main
struct OffriiApp: App {
    @State private var authManager = AuthManager()
    @State private var router = AppRouter()
    @AppStorage("hasSeenOnboarding") private var hasSeenOnboarding = false

    var body: some Scene {
        WindowGroup {
            Group {
                switch router.currentScreen {
                case .onboarding:
                    OnboardingView {
                        hasSeenOnboarding = true
                        router.determineInitialScreen(
                            isAuthenticated: authManager.isAuthenticated,
                            hasSeenOnboarding: true
                        )
                    } onSignIn: {
                        hasSeenOnboarding = true
                        router.currentScreen = .auth
                    }

                case .auth:
                    AuthContainerView {
                        router.currentScreen = .main
                    }
                    .environment(authManager)

                case .main:
                    MainTabView()
                        .environment(authManager)
                        .onChange(of: authManager.currentUser == nil) { _, isNil in
                            if isNil && !authManager.isAuthenticated {
                                router.currentScreen = .auth
                            }
                        }
                }
            }
            .onAppear {
                router.determineInitialScreen(
                    isAuthenticated: authManager.isAuthenticated,
                    hasSeenOnboarding: hasSeenOnboarding
                )
            }
        }
    }
}

struct AuthContainerView: View {
    @State private var showLogin = true
    let onAuthenticated: () -> Void

    var body: some View {
        if showLogin {
            LoginView(onAuthenticated: onAuthenticated, onSwitchToRegister: { showLogin = false })
        } else {
            RegisterView(onAuthenticated: onAuthenticated, onSwitchToLogin: { showLogin = true })
        }
    }
}

struct MainTabView: View {
    @State private var selectedTab: TabItem = .envies

    var body: some View {
        VStack(spacing: 0) {
            Group {
                switch selectedTab {
                case .envies:
                    NavigationStack {
                        WishlistView()
                    }
                case .cercles:
                    NavigationStack {
                        CirclesListView()
                    }
                case .entraide:
                    ComingSoonView(icon: "hand.raised.fill", featureName: "Entraide")
                case .profil:
                    NavigationStack {
                        ProfileView()
                    }
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)

            TabBarView(selectedTab: $selectedTab)
        }
        .ignoresSafeArea(.keyboard)
    }
}
