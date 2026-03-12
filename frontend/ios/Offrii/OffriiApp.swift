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
                case .splash:
                    SplashView()

                case .welcome:
                    WelcomeView()

                case .auth:
                    AuthContainerView { isNewUser in
                        if isNewUser {
                            router.currentScreen = .postAuthSetup
                        } else {
                            router.currentScreen = .main
                        }
                    }

                case .postAuthSetup:
                    PostAuthSetupView {
                        router.completePostAuthSetup()
                    }

                case .main:
                    MainTabView()
                        .environment(tipManager)
                        .onChange(of: authManager.currentUser) { _, user in
                            if user == nil {
                                router.preferRegister = false
                                router.currentScreen = .auth
                            }
                        }
                }
            }
            .animation(OffriiAnimation.modal, value: router.currentScreen)
            .environment(authManager)
            .environment(router)
        }
    }
}

struct AuthContainerView: View {
    @Environment(AppRouter.self) private var router
    @State private var showLogin: Bool?
    let onAuthenticated: (_ isNewUser: Bool) -> Void

    private var isLogin: Bool { showLogin ?? !router.preferRegister }

    var body: some View {
        if isLogin {
            LoginView(
                isReturningUser: router.isReturningUser,
                onAuthenticated: { onAuthenticated(false) },
                onSwitchToRegister: { showLogin = false }
            )
        } else {
            RegisterView(
                onAuthenticated: { onAuthenticated(true) },
                onSwitchToLogin: { showLogin = true }
            )
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

