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
                onAuthenticated: { isNewUser in onAuthenticated(isNewUser) },
                onSwitchToRegister: { showLogin = false }
            )
        } else {
            RegisterView(
                onAuthenticated: { isNewUser in onAuthenticated(isNewUser) },
                onSwitchToLogin: { showLogin = true }
            )
        }
    }
}

struct MainTabView: View {
    @State private var selectedTab: TabItem = .home
    @State private var showCreateSheet = false

    var body: some View {
        VStack(spacing: 0) {
            Group {
                switch selectedTab {
                case .home:
                    NavigationStack {
                        HomeView()
                    }
                case .envies:
                    NavigationStack {
                        WishlistView()
                    }
                case .create:
                    EmptyView()
                case .cercles:
                    NavigationStack {
                        CirclesListView()
                    }
                case .entraide:
                    NavigationStack {
                        EntraideView()
                    }
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)

            TabBarView(selectedTab: $selectedTab, onCreateTap: {
                showCreateSheet = true
            })
        }
        .ignoresSafeArea(.keyboard)
        .sheet(isPresented: $showCreateSheet) {
            QuickCreateSheet()
                .presentationDetents([.medium])
        }
    }
}
