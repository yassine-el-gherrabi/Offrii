import SwiftUI

@main
struct OffriiApp: App {
    @State private var authManager = AuthManager()
    @State private var router = AppRouter()
    @State private var tipManager = OnboardingTipManager()

    var body: some Scene {
        WindowGroup {
            Group {
                switch router.currentScreen {
                case .auth:
                    AuthContainerView {
                        router.currentScreen = .main
                    }
                    .environment(authManager)

                case .main:
                    MainTabView()
                        .environment(authManager)
                        .environment(tipManager)
                        .onChange(of: authManager.currentUser) { _, user in
                            if user == nil {
                                router.currentScreen = .auth
                            }
                        }
                }
            }
            .onAppear {
                router.determineInitialScreen(
                    isAuthenticated: authManager.isAuthenticated
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
                    NavigationStack {
                        EntraideView()
                    }
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
