import SwiftUI

// MARK: - EntraideView

struct EntraideView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = EntraideViewModel()
    @State private var myNeedsViewModel = EntraideMyNeedsViewModel()
    @State private var selectedSegment = 0
    @State private var showCreateSheet = false
    @State private var selectedWishId: UUID?
    @State private var messagesWishId: UUID?
    @State private var reportWishId: UUID?
    @State private var searchQuery = ""
    @State private var showWishLimitAlert = false
    @State private var showEligibilityAlert = false
    @State private var resendCooldown = false
    @State private var resendCountdown = 0
    @State private var resendError: String?
    @State private var sortField = "created_at"
    @State private var sortOrder = "desc"
    @State private var showWelcomeSheet = false

    private var isAccountTooRecent: Bool {
        guard let user = authManager.currentUser else { return true }
        return Date().timeIntervalSince(user.createdAt) < 24 * 3600
    }

    private var isEmailVerified: Bool {
        authManager.currentUser?.emailVerified ?? true
    }

    private var isEligible: Bool {
        !isAccountTooRecent && isEmailVerified
    }

    private var eligibilityMessage: String {
        if isAccountTooRecent {
            return NSLocalizedString("entraide.eligibility.tooRecent", comment: "")
        }
        return NSLocalizedString("entraide.eligibility.emailNotVerified", comment: "")
    }

    private var segmentLabel: String {
        switch selectedSegment {
        case 0:  return NSLocalizedString("entraide.segment.discover", comment: "")
        case 1:  return NSLocalizedString("entraide.segment.myNeeds", comment: "")
        default: return NSLocalizedString("entraide.segment.myOffers", comment: "")
        }
    }

    private var isCurrentSegmentLoading: Bool {
        switch selectedSegment {
        case 0:  return viewModel.isLoading
        case 1:  return myNeedsViewModel.isLoading
        default: return viewModel.isLoading
        }
    }

    private var displayCount: Int {
        switch selectedSegment {
        case 0:  return viewModel.filteredWishes.count
        case 1:  return myNeedsViewModel.wishes.count
        default: return viewModel.myOfferWishes.count
        }
    }

    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            ScrollView {
                LazyVStack(spacing: 0, pinnedViews: .sectionHeaders) {
                    Section {
                        switch selectedSegment {
                        case 0:
                            EntraideDiscoverContent(
                                viewModel: viewModel,
                                selectedWishId: $selectedWishId,
                                showCreateSheet: $showCreateSheet,
                                reportWishId: $reportWishId,
                                searchQuery: searchQuery
                            )
                        case 1:
                            EntraideMyNeedsContent(
                                viewModel: myNeedsViewModel,
                                selectedWishId: $selectedWishId,
                                showCreateSheet: $showCreateSheet
                            )
                        case 2:
                            EntraideMyOffersContent(
                                viewModel: viewModel,
                                selectedWishId: $selectedWishId,
                                messagesWishId: $messagesWishId
                            )
                        default:
                            EmptyView()
                        }
                    } header: {
                        VStack(spacing: 0) {
                            Text(NSLocalizedString("entraide.subtitle", comment: ""))
                                .font(.system(size: 13, weight: .regular).italic())
                                .foregroundColor(OffriiTheme.textSecondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding(.horizontal, OffriiTheme.spacingBase)
                                .padding(.bottom, 2)

                            // Eligibility banner
                            if !isEligible {
                                eligibilityBanner
                            }

                            categoryChipsBar
                            statsBar
                        }
                        .background(OffriiTheme.background)
                    }
                }
            }
            .refreshable {
                switch selectedSegment {
                case 0:  await viewModel.loadWishes()
                case 1:  await myNeedsViewModel.loadMyWishes()
                default: await viewModel.loadMyOffers()
                }
            }

            // FAB (hidden when account too recent, shows alert when email not verified)
            if (selectedSegment == 0 || selectedSegment == 1) && !isAccountTooRecent {
                OffriiFloatingActionButton(icon: "plus") {
                    if !isEmailVerified {
                        showEligibilityAlert = true
                        return
                    }
                    let activeCount = myNeedsViewModel.wishes.filter {
                        $0.status == .open || $0.status == .matched || $0.status == .pending
                    }.count
                    if activeCount >= 3 {
                        showWishLimitAlert = true
                    } else {
                        showCreateSheet = true
                    }
                }
                .padding(.trailing, OffriiTheme.spacingLG)
                .padding(.bottom, OffriiTheme.spacingLG)
            }
        }
        .background(OffriiTheme.background.ignoresSafeArea())
        .navigationTitle(NSLocalizedString("entraide.title", comment: ""))
        .navigationBarTitleDisplayMode(.large)
        .searchable(
            text: $searchQuery,
            placement: .navigationBarDrawer(displayMode: .automatic),
            prompt: NSLocalizedString("entraide.search.placeholder", comment: "")
        )
        .toolbar {
            ToolbarItemGroup(placement: .topBarTrailing) {
                NavigationLink(destination: ProfileView()) {
                    ProfileAvatarButton(
                        initials: ProfileAvatarButton.initials(
                            from: authManager.currentUser?.displayName
                        ),
                        avatarUrl: authManager.currentUser?.avatarUrl
                            .flatMap { URL(string: $0) }
                    )
                }
            }
        }
        .sheet(isPresented: $showCreateSheet, onDismiss: {
            Task {
                await viewModel.loadWishes()
                await viewModel.loadMyOffers()
                await myNeedsViewModel.loadMyWishes()
            }
        }) {
            CreateWishSheet()
                .presentationDetents([.large])
        }
        .sheet(item: $selectedWishId, onDismiss: {
            Task {
                await viewModel.loadWishes()
                await viewModel.loadMyOffers()
                await myNeedsViewModel.loadMyWishes()
            }
        }) { wishId in
            WishDetailSheet(
                wishId: wishId,
                onOpenMessages: { messagesWishId = wishId },
                onReport: { reportWishId = wishId },
                onAction: {
                    Task {
                        await viewModel.loadWishes()
                        await viewModel.loadMyOffers()
                        await myNeedsViewModel.loadMyWishes()
                    }
                }
            )
            .environment(authManager)
            .presentationDetents([.medium, .large])
        }
        .sheet(item: $messagesWishId) { wishId in
            WishMessagesSheet(wishId: wishId)
                .presentationDetents([.large])
        }
        .sheet(item: $reportWishId, onDismiss: {
            Task { await viewModel.loadWishes() }
        }) { wishId in
            ReportWishSheet(wishId: wishId)
                .presentationDetents([.medium])
        }
        .alert(
            NSLocalizedString("entraide.wishLimit.title", comment: ""),
            isPresented: $showWishLimitAlert
        ) {
            Button(NSLocalizedString("common.ok", comment: ""), role: .cancel) {}
        } message: {
            Text(NSLocalizedString("entraide.wishLimit.message", comment: ""))
        }
        .alert(
            NSLocalizedString("entraide.eligibility.emailNotVerified", comment: ""),
            isPresented: $showEligibilityAlert
        ) {
            Button(NSLocalizedString("entraide.eligibility.resend", comment: "")) {
                Task { await resendVerificationWithCooldown() }
            }
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {}
        }
        .task {
            await viewModel.loadWishes()
            await viewModel.loadMyOffers()
            await myNeedsViewModel.loadMyWishes()
            if let userId = authManager.currentUser?.id.uuidString {
                let key = "entraide.hasVisited.\(userId)"
                if !UserDefaults.standard.bool(forKey: key) {
                    showWelcomeSheet = true
                }
                UserDefaults.standard.set(true, forKey: key)
            }
        }
        .sheet(isPresented: $showWelcomeSheet) {
            EntraideWelcomeSheet()
                .presentationDetents([.medium])
        }
    }

    // MARK: - Category Chips (same as WishlistView pattern)

    private var categoryChipsBar: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                let allSelected = viewModel.selectedCategory == nil

                Button {
                    Task { await viewModel.selectCategory(nil) }
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "sparkles")
                            .font(.system(size: 11))
                        Text(NSLocalizedString("entraide.category.all", comment: ""))
                            .font(.system(size: 13, weight: allSelected ? .semibold : .regular))
                    }
                    .foregroundColor(allSelected ? .white : OffriiTheme.textSecondary)
                    .padding(.horizontal, OffriiTheme.spacingMD)
                    .padding(.vertical, OffriiTheme.spacingSM)
                    .background(allSelected ? OffriiTheme.primary : .white)
                    .cornerRadius(OffriiTheme.cornerRadiusXL)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                            .strokeBorder(allSelected ? .clear : OffriiTheme.border, lineWidth: 1)
                    )
                }
                .buttonStyle(.plain)

                ForEach(WishCategory.allCases) { category in
                    let isSelected = viewModel.selectedCategory == category
                    let color = category.color

                    HStack(spacing: 4) {
                        Image(systemName: category.icon)
                            .font(.system(size: 11))
                        Text(category.label)
                            .font(.system(size: 13, weight: isSelected ? .semibold : .regular))
                    }
                    .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
                    .padding(.horizontal, OffriiTheme.spacingMD)
                    .padding(.vertical, OffriiTheme.spacingSM)
                    .background(isSelected ? color : .white)
                    .cornerRadius(OffriiTheme.cornerRadiusXL)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                            .strokeBorder(isSelected ? .clear : OffriiTheme.border, lineWidth: 1)
                    )
                    .onTapGesture {
                        Task { await viewModel.selectCategory(category) }
                    }
                    .animation(OffriiAnimation.snappy, value: isSelected)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingXS)
        }
    }

    // MARK: - Stats Bar (same pattern as WishlistView)

    private var statsBar: some View {
        HStack {
            HStack(spacing: 4) {
                if isCurrentSegmentLoading {
                    RoundedRectangle(cornerRadius: 3)
                        .fill(OffriiTheme.textMuted.opacity(0.2))
                        .frame(width: 20, height: 14)
                        .shimmer()
                } else {
                    Text("\(displayCount)")
                        .font(.system(size: 13, weight: .medium))
                        .foregroundColor(OffriiTheme.text)
                }
                Text(displayCount == 1
                    ? NSLocalizedString("entraide.countSingular", comment: "")
                    : NSLocalizedString("entraide.countPlural", comment: ""))
                    .font(.system(size: 13))
                    .foregroundColor(OffriiTheme.textMuted)

                Text("·").foregroundColor(OffriiTheme.textMuted)

                SortMenuView(
                    options: [
                        ("created_at", NSLocalizedString("entraide.sort.date", comment: "")),
                        ("title", NSLocalizedString("entraide.sort.name", comment: "")),
                    ],
                    sortField: $sortField,
                    sortOrder: $sortOrder
                )
                .onChange(of: sortField) { _, _ in applySort() }
                .onChange(of: sortOrder) { _, _ in applySort() }
            }

            Spacer()

            Picker("", selection: $selectedSegment) {
                Text(NSLocalizedString("entraide.segment.discover", comment: "")).tag(0)
                Text(NSLocalizedString("entraide.segment.myNeeds", comment: "")).tag(1)
                Text(NSLocalizedString("entraide.segment.myOffers", comment: "")).tag(2)
            }
            .pickerStyle(.segmented)
            .frame(width: 260)
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingXS)
    }

    // MARK: - Sort

    private func applySort() {
        viewModel.sortField = sortField
        viewModel.sortOrder = sortOrder
    }

    // MARK: - Eligibility Banner

    private var eligibilityBanner: some View {
        VStack(spacing: OffriiTheme.spacingSM) {
            HStack(spacing: OffriiTheme.spacingSM) {
                Image(systemName: isAccountTooRecent ? "clock" : "envelope.badge")
                    .font(.system(size: 14))
                    .foregroundColor(OffriiTheme.primary)

                Text(eligibilityMessage)
                    .font(OffriiTypography.caption)
                    .foregroundColor(OffriiTheme.text)

                Spacer()

                if !isAccountTooRecent && !isEmailVerified {
                    Button {
                        Task { await resendVerificationWithCooldown() }
                    } label: {
                        if resendCooldown {
                            Text("\(resendCountdown)s")
                                .font(.system(size: 12, weight: .medium).monospacedDigit())
                                .foregroundColor(OffriiTheme.textMuted)
                        } else {
                            Text(NSLocalizedString("entraide.eligibility.resend", comment: ""))
                                .font(.system(size: 12, weight: .semibold))
                                .foregroundColor(OffriiTheme.primary)
                        }
                    }
                    .disabled(resendCooldown)
                }
            }

            // Success/error feedback
            if resendCooldown && resendCountdown > 50 {
                Text(NSLocalizedString("entraide.eligibility.checkSpam", comment: ""))
                    .font(.system(size: 11))
                    .foregroundColor(OffriiTheme.textSecondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }

            if let resendError {
                Text(resendError)
                    .font(.system(size: 11))
                    .foregroundColor(OffriiTheme.danger)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .padding(OffriiTheme.spacingSM)
        .padding(.horizontal, OffriiTheme.spacingXS)
        .background(OffriiTheme.primary.opacity(0.08))
        .cornerRadius(OffriiTheme.cornerRadiusMD)
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingXS)
    }

    // MARK: - Resend Verification

    private func resendVerificationWithCooldown() async {
        guard !resendCooldown else { return }
        resendError = nil
        do {
            try await UserService.shared.resendVerification()
            OffriiHaptics.success()
            startCooldown(seconds: 60)
        } catch let error as APIError {
            if case .tooManyRequests = error {
                startCooldown(seconds: 60)
            } else {
                resendError = error.localizedDescription
                OffriiHaptics.error()
            }
        } catch {
            resendError = error.localizedDescription
            OffriiHaptics.error()
        }
    }

    private func startCooldown(seconds: Int) {
        resendCooldown = true
        resendCountdown = seconds
        Task {
            while resendCountdown > 0 {
                try? await Task.sleep(for: .seconds(1))
                resendCountdown -= 1
            }
            resendCooldown = false
        }
    }

}
