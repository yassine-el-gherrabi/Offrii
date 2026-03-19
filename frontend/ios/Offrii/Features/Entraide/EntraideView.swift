import SwiftUI

// MARK: - EntraideView

struct EntraideView: View {
    @Environment(AuthManager.self) private var authManager
    @State private var viewModel = EntraideViewModel()
    @State private var selectedSegment = 0
    @State private var showCreateSheet = false
    @State private var selectedWishId: UUID?
    @State private var messagesWishId: UUID?
    @State private var reportWishId: UUID?

    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            VStack(spacing: 0) {
                // Segment picker (like Proches filter chips style)
                segmentChips

                switch selectedSegment {
                case 0:
                    EntraideDiscoverContent(
                        viewModel: viewModel,
                        selectedWishId: $selectedWishId
                    )
                case 1:
                    EntraideMyNeedsContent(
                        selectedWishId: $selectedWishId,
                        showCreateSheet: $showCreateSheet
                    )
                case 2:
                    EntraideMyOffersContent(
                        viewModel: viewModel,
                        selectedWishId: $selectedWishId
                    )
                default:
                    EmptyView()
                }
            }

            // FAB — only on Discover
            if selectedSegment == 0 {
                OffriiFloatingActionButton(
                    icon: "plus",
                    label: NSLocalizedString("entraide.fab.publish", comment: "")
                ) {
                    showCreateSheet = true
                }
                .padding(.trailing, OffriiTheme.spacingLG)
                .padding(.bottom, OffriiTheme.spacingLG)
            }
        }
        .background(OffriiTheme.background.ignoresSafeArea())
        .navigationTitle(NSLocalizedString("entraide.title", comment: ""))
        .navigationBarTitleDisplayMode(.large)
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
            Task { await viewModel.loadWishes() }
        }) {
            CreateWishSheet()
                .presentationDetents([.large])
        }
        .sheet(item: $selectedWishId, onDismiss: {
            Task { await viewModel.loadWishes() }
        }) { wishId in
            WishDetailSheet(
                wishId: wishId,
                onOpenMessages: { messagesWishId = wishId },
                onReport: { reportWishId = wishId }
            )
            .environment(authManager)
            .presentationDetents([.medium, .large])
        }
        .sheet(item: $messagesWishId) { wishId in
            WishMessagesSheet(wishId: wishId)
                .presentationDetents([.large])
        }
        .sheet(item: $reportWishId) { wishId in
            ReportWishSheet(wishId: wishId)
                .presentationDetents([.medium])
        }
        .task {
            await viewModel.loadWishes()
        }
    }

    // MARK: - Segment Chips (same pattern as Proches filterChips)

    private var segmentChips: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                OffriiChip(
                    title: NSLocalizedString("entraide.segment.discover", comment: ""),
                    isSelected: selectedSegment == 0,
                    action: { selectedSegment = 0 }
                )
                OffriiChip(
                    title: NSLocalizedString("entraide.segment.myNeeds", comment: ""),
                    isSelected: selectedSegment == 1,
                    action: { selectedSegment = 1 }
                )
                OffriiChip(
                    title: NSLocalizedString("entraide.segment.myOffers", comment: ""),
                    isSelected: selectedSegment == 2,
                    action: { selectedSegment = 2 }
                )
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
        }
        .padding(.vertical, OffriiTheme.spacingSM)
    }
}
