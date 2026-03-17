import SwiftUI
import UserNotifications

@main
struct OffriiApp: App {
    @UIApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
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
            .onOpenURL { url in
                router.handleURL(url)
            }
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
    @Environment(AppRouter.self) private var router
    @State private var selectedTab: TabItem = .home
    @State private var showCreateSheet = false
    @State private var joinResult: String?
    @State private var joinError: String?

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
        .task {
            let center = UNUserNotificationCenter.current()
            let settings = await center.notificationSettings()
            if settings.authorizationStatus == .authorized {
                await MainActor.run {
                    UIApplication.shared.registerForRemoteNotifications()
                }
            }
        }
        .sheet(isPresented: $showCreateSheet) {
            QuickCreateSheet()
                .presentationDetents([.medium])
        }
        .onChange(of: router.pendingInviteToken) { _, token in
            guard let token else { return }
            Task {
                do {
                    let result = try await CircleService.shared.joinViaInvite(token: token)
                    joinResult = result.circleName ?? NSLocalizedString("circles.unnamed", comment: "")
                    OffriiHaptics.success()
                } catch {
                    joinError = error.localizedDescription
                }
                router.pendingInviteToken = nil
            }
        }
        .alert(
            NSLocalizedString("circles.invite.joined", comment: ""),
            isPresented: Binding(
                get: { joinResult != nil },
                set: { if !$0 { joinResult = nil } }
            )
        ) {
            Button(NSLocalizedString("common.ok", comment: "")) {
                selectedTab = .cercles
                joinResult = nil
            }
        } message: {
            if let name = joinResult {
                Text(String(format: NSLocalizedString("circles.invite.joinedMessage", comment: ""), name))
            }
        }
        .alert(
            NSLocalizedString("common.error", comment: ""),
            isPresented: Binding(
                get: { joinError != nil },
                set: { if !$0 { joinError = nil } }
            )
        ) {
            Button(NSLocalizedString("common.ok", comment: ""), role: .cancel) {}
        } message: {
            if let err = joinError { Text(err) }
        }
    }
}
