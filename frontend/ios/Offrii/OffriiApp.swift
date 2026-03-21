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
                    AuthView(
                        initialMode: router.preferRegister ? .register : .login
                    ) { isNewUser in
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
            .onAppear {
                appDelegate.router = router
            }
            .onOpenURL { url in
                router.handleURL(url)
            }
        }
    }
}

struct MainTabView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(AppRouter.self) private var router
    @State private var selectedTab: TabItem = .home
    @State private var showCreateSheet = false
    @State private var joinResult: String?
    @State private var joinError: String?
    @State private var circlesPath = NavigationPath()

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
                    NavigationStack(path: $circlesPath) {
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
        .onReceive(NotificationCenter.default.publisher(for: UIApplication.willEnterForegroundNotification)) { _ in
            Task {
                let center = UNUserNotificationCenter.current()
                let settings = await center.notificationSettings()
                if settings.authorizationStatus == .authorized {
                    await MainActor.run {
                        UIApplication.shared.registerForRemoteNotifications()
                    }
                }
                // Refresh app icon badge count
                await AppDelegate.refreshBadgeCount()
                // Refresh current user (picks up email verification, etc.)
                try? await authManager.loadCurrentUser()
            }
        }
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
        .onChange(of: router.selectedTab) { _, tab in
            if let tab {
                selectedTab = tab
                router.selectedTab = nil
            }
        }
        .onChange(of: router.pendingCircleId) { _, circleId in
            guard let circleId else { return }
            selectedTab = .cercles
            // Small delay to ensure tab switch completes before pushing
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                circlesPath.append(circleId)
                router.pendingCircleId = nil
            }
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
        .onChange(of: router.showFriends) { _, show in
            guard show else { return }
            selectedTab = .cercles
            // Don't reset here — CirclesListView will consume it and switch to friends filter
        }
    }
}
