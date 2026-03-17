import SwiftUI

// swiftlint:disable:next type_body_length
struct WishlistView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(OnboardingTipManager.self) private var tipManager
    @State private var viewModel = WishlistViewModel()
    @State private var showQuickAdd = false
    @State private var selectedItemId: UUID?
    @State private var showShareSheet = false
    @State private var shareToCircleItemId: UUID?
    @State private var editItem: Item?
    @State private var showBatchDeleteConfirm = false
    @State private var shareItemId: UUID?

    private let gridColumns = [
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM),
        GridItem(.flexible(), spacing: OffriiTheme.spacingSM)
    ]

    var body: some View {
        ZStack(alignment: .bottomTrailing) {
            ScrollView {
                LazyVStack(spacing: 0, pinnedViews: .sectionHeaders) {
                    Section {
                        gridInnerContent
                    } header: {
                        VStack(spacing: 0) {
                            categoryChipsBar
                            statsBar
                        }
                        .background(OffriiTheme.background)
                    }
                }
            }
            .refreshable {
                await viewModel.loadItems()
            }

            if !viewModel.isSelectMode {
                OffriiFloatingActionButton(icon: "plus") {
                    showQuickAdd = true
                }
                .padding(.trailing, OffriiTheme.spacingLG)
                .padding(.bottom, OffriiTheme.spacingLG)
            }

            if viewModel.isSelectMode {
                VStack {
                    Spacer()
                    batchActionBar
                }
            }
        }
        .background(OffriiTheme.background.ignoresSafeArea())
        .navigationTitle(NSLocalizedString("wishlist.title", comment: ""))
        .navigationBarTitleDisplayMode(.large)
        .searchable(
            text: $viewModel.searchQuery,
            placement: .navigationBarDrawer(displayMode: .automatic),
            prompt: NSLocalizedString("wishlist.search.placeholder", comment: "")
        )
        .toolbar {
            ToolbarItemGroup(placement: .topBarTrailing) {
                if viewModel.isSelectMode {
                    Button {
                        viewModel.selectAllVisible()
                    } label: {
                        Text(NSLocalizedString("wishlist.selectAll", comment: ""))
                            .font(.system(size: 14, weight: .medium))
                    }

                    Button {
                        viewModel.toggleSelectMode()
                    } label: {
                        Text(NSLocalizedString("common.cancel", comment: ""))
                            .font(.system(size: 14, weight: .medium))
                    }
                } else {
                    Menu {
                        Button {
                            viewModel.toggleSelectMode()
                        } label: {
                            Label(NSLocalizedString("wishlist.select", comment: ""), systemImage: "checklist")
                        }

                        Button {
                            showShareSheet = true
                        } label: {
                            Label(NSLocalizedString("wishlist.share", comment: ""), systemImage: "square.and.arrow.up")
                        }
                    } label: {
                        Image(systemName: "ellipsis.circle")
                            .font(.system(size: 18))
                            .foregroundColor(OffriiTheme.primary)
                    }

                    NavigationLink(destination: ProfileView()) {
                        ProfileAvatarButton(
                            initials: ProfileAvatarButton.initials(from: authManager.currentUser?.displayName),
                            avatarUrl: authManager.currentUser?.avatarUrl.flatMap { URL(string: $0) }
                        )
                    }
                }
            }
        }
        .sheet(isPresented: $showQuickAdd) {
            QuickAddSheet { name, price, categoryId, priority, imageUrl, links in
                await viewModel.quickAdd(name: name, price: price, categoryId: categoryId, priority: priority, imageUrl: imageUrl, links: links)
            }
        }
        .sheet(item: $selectedItemId, onDismiss: {
            Task { await viewModel.loadItems() }
        }) { itemId in
            ItemDetailSheet(itemId: itemId)
                .environment(authManager)
                .presentationDetents([.medium, .large])
        }
        .sheet(isPresented: $showShareSheet) {
            WishlistShareSheet(
                items: viewModel.items,
                selectedItemIds: viewModel.selectedItemIds,
                privateItemCount: viewModel.items.filter(\.isPrivate).count
            )
            .presentationDetents([.large])
        }
        .sheet(item: $shareToCircleItemId) { itemId in
            let alreadyShared = Set(
                viewModel.items.first(where: { $0.id == itemId })?.sharedCircles.map(\.id) ?? []
            )
            ShareToCircleSheet(itemId: itemId, alreadySharedCircleIds: alreadyShared)
                .presentationDetents([.medium])
        }
        .sheet(item: $editItem) { item in
            NavigationStack {
                ItemEditView(item: item) { _ in
                    Task { await viewModel.loadItems() }
                }
            }
        }
        .sheet(item: $shareItemId) { itemId in
            ShareItemSheet(itemId: itemId)
                .presentationDetents([.medium])
        }
        .alert(
            NSLocalizedString("wishlist.deleteConfirmBatch", comment: ""),
            isPresented: $showBatchDeleteConfirm
        ) {
            Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {}
            Button(
                String(format: NSLocalizedString("wishlist.deleteCount", comment: ""), viewModel.selectedItemIds.count),
                role: .destructive
            ) {
                Task { await viewModel.batchDelete() }
            }
        } message: {
            Text(String(format: NSLocalizedString("wishlist.deleteConfirmBatchMessage", comment: ""), viewModel.selectedItemIds.count))
        }
        .task {
            await viewModel.loadCategories()
            await viewModel.loadItems()
            if viewModel.items.isEmpty {
                tipManager.showIfNeeded(.wishlistFirstAdd)
            }
        }
        .onAppear {
            // Reload when returning from another tab to pick up share changes
            if !viewModel.items.isEmpty {
                Task { await viewModel.loadItems() }
            }
        }
    }

    // MARK: - Category Chips

    private var categoryChipsBar: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                let allSelected = viewModel.selectedCategoryIds.isEmpty
                Button {
                    viewModel.clearCategoryFilters()
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "sparkles")
                            .font(.system(size: 11))
                        Text(NSLocalizedString("wishlist.allCategories", comment: ""))
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

                ForEach(viewModel.categories, id: \.id) { cat in
                    let isSelected = viewModel.selectedCategoryIds.contains(cat.id)
                    let style = CategoryStyle(icon: cat.icon)

                    HStack(spacing: 4) {
                        Image(systemName: style.sfSymbol)
                            .font(.system(size: 11))
                        Text(cat.name)
                            .font(.system(size: 13, weight: isSelected ? .semibold : .regular))
                    }
                    .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
                    .padding(.horizontal, OffriiTheme.spacingMD)
                    .padding(.vertical, OffriiTheme.spacingSM)
                    .background(isSelected ? style.chipColor : .white)
                    .cornerRadius(OffriiTheme.cornerRadiusXL)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                            .strokeBorder(isSelected ? .clear : OffriiTheme.border, lineWidth: 1)
                    )
                    .onTapGesture {
                        viewModel.toggleCategory(cat.id)
                    }
                    .onLongPressGesture(minimumDuration: 0.5) {
                        OffriiHaptics.tap()
                        viewModel.selectOnlyCategory(cat.id)
                    }
                    .animation(OffriiAnimation.snappy, value: isSelected)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingXS)
        }
    }

    // MARK: - Stats Bar

    private var statsBar: some View {
        HStack {
            HStack(spacing: 4) {
                Text("\(viewModel.filteredItems.count)")
                    .font(.system(size: 13, weight: .medium))
                    .foregroundColor(OffriiTheme.text)
                Text(viewModel.filteredItems.count == 1
                     ? NSLocalizedString("wishlist.countSingular", comment: "")
                     : NSLocalizedString("wishlist.countPlural", comment: ""))
                    .font(.system(size: 13))
                    .foregroundColor(OffriiTheme.textMuted)

                Text("·").foregroundColor(OffriiTheme.textMuted)

                Menu {
                    Button { viewModel.changeSort("created_at") } label: {
                        Label(NSLocalizedString("wishlist.sort.date", comment: ""),
                              systemImage: viewModel.sortField == "created_at" ? "checkmark" : "")
                    }
                    Button { viewModel.changeSort("priority") } label: {
                        Label(NSLocalizedString("wishlist.sort.priority", comment: ""),
                              systemImage: viewModel.sortField == "priority" ? "checkmark" : "")
                    }
                    Button { viewModel.changeSort("name") } label: {
                        Label(NSLocalizedString("wishlist.sort.name", comment: ""),
                              systemImage: viewModel.sortField == "name" ? "checkmark" : "")
                    }
                } label: {
                    HStack(spacing: 2) {
                        Text(sortLabel)
                            .font(.system(size: 13, weight: .medium))
                        Image(systemName: viewModel.sortOrder == "asc" ? "arrow.up" : "arrow.down")
                            .font(.system(size: 10, weight: .semibold))
                    }
                    .foregroundColor(OffriiTheme.primary)
                }
            }

            Spacer()

            Picker("", selection: Binding(
                get: { viewModel.filteredSegmentIndex },
                set: { viewModel.changeSegment($0) }
            )) {
                Text(NSLocalizedString("wishlist.active", comment: "")).tag(0)
                Text(NSLocalizedString("wishlist.reserved", comment: "")).tag(1)
                Text(NSLocalizedString("wishlist.purchased", comment: "")).tag(2)
            }
            .pickerStyle(.segmented)
            .frame(width: 240)
        }
        .padding(.horizontal, OffriiTheme.spacingBase)
        .padding(.vertical, OffriiTheme.spacingXS)
    }

    // MARK: - Grid Inner Content (no ScrollView wrapper)

    @ViewBuilder
    private var gridInnerContent: some View {
        let displayItems = viewModel.filteredItems
        let isPurchasedTab = viewModel.selectedStatus == "purchased"

        if viewModel.isLoading && viewModel.items.isEmpty {
            LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                ForEach(0..<6, id: \.self) { _ in SkeletonGridCard() }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
        } else if displayItems.isEmpty {
            VStack(spacing: 0) {
                Spacer().frame(height: 80)
                OffriiEmptyState(
                    icon: "gift",
                    title: NSLocalizedString("wishlist.empty", comment: ""),
                    subtitle: NSLocalizedString("wishlist.emptySubtitle", comment: ""),
                    ctaTitle: viewModel.searchQuery.isEmpty ? NSLocalizedString("wishlist.quickAdd.button", comment: "") : nil,
                    ctaAction: { showQuickAdd = true }
                )
                Spacer().frame(height: 80)
            }
        } else {
            LazyVGrid(columns: gridColumns, spacing: OffriiTheme.spacingSM) {
                ForEach(displayItems) { item in
                    ZStack(alignment: .topLeading) {
                        WishlistGridCard(
                            item: item,
                            category: viewModel.category(for: item.categoryId),
                            isPurchasedTab: isPurchasedTab
                        ) {
                            if viewModel.isSelectMode {
                                viewModel.toggleItemSelection(item.id)
                            } else {
                                selectedItemId = item.id
                            }
                        }

                        if viewModel.isSelectMode {
                            let isChecked = viewModel.selectedItemIds.contains(item.id)
                            Image(systemName: isChecked ? "checkmark.circle.fill" : "circle")
                                .font(.system(size: 22))
                                .symbolRenderingMode(.hierarchical)
                                .foregroundColor(isChecked ? OffriiTheme.primary : .white)
                                .shadow(color: .black.opacity(0.3), radius: 2, x: 0, y: 1)
                                .padding(OffriiTheme.spacingSM)
                                .animation(OffriiAnimation.micro, value: isChecked)
                        }
                    }
                    .onAppear {
                        Task { await viewModel.loadMoreIfNeeded(currentItem: item) }
                    }
                    .contextMenu {
                        if !viewModel.isSelectMode {
                            if !item.isPrivate {
                                Button {
                                    shareToCircleItemId = item.id
                                } label: {
                                    Label(NSLocalizedString("share.addPeople", comment: ""), systemImage: "person.2")
                                }

                                Button {
                                    shareItemId = item.id
                                } label: {
                                    Label(NSLocalizedString("share.item", comment: ""), systemImage: "square.and.arrow.up")
                                }

                                Divider()
                            }

                            Button {
                                editItem = item
                            } label: {
                                Label(NSLocalizedString("wishlist.edit", comment: ""), systemImage: "pencil")
                            }

                            if item.isActive {
                                Button {
                                    Task { await viewModel.markPurchased(item) }
                                } label: {
                                    Label(
                                        item.isClaimed
                                            ? NSLocalizedString("wishlist.receivedGift", comment: "")
                                            : NSLocalizedString("wishlist.markReceived", comment: ""),
                                        systemImage: "checkmark.circle"
                                    )
                                }
                            }

                            if item.isPurchased {
                                Button {
                                    Task { await viewModel.unarchiveItem(item) }
                                } label: {
                                    Label(NSLocalizedString("wishlist.unarchive", comment: ""), systemImage: "arrow.uturn.backward")
                                }
                            }

                            Button {
                                UIPasteboard.general.string = item.name
                            } label: {
                                Label(NSLocalizedString("wishlist.copyName", comment: ""), systemImage: "doc.on.doc")
                            }

                            Divider()

                            Button(role: .destructive) {
                                Task { await viewModel.deleteItem(item) }
                            } label: {
                                Label(NSLocalizedString("common.delete", comment: ""), systemImage: "trash")
                            }
                        }
                    }
                }

                if viewModel.isLoadingMore {
                    SkeletonGridCard()
                    SkeletonGridCard()
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
            .padding(.bottom, viewModel.isSelectMode ? 70 : 0)
        }
    }

    // MARK: - Batch Action Bar

    private var batchActionBar: some View {
        HStack {
            Text("\(viewModel.selectedItemIds.count) " + NSLocalizedString("wishlist.selected", comment: ""))
                .font(.system(size: 14, weight: .medium))
                .foregroundColor(OffriiTheme.text)

            Spacer()

            Button {
                showBatchDeleteConfirm = true
            } label: {
                HStack(spacing: 4) {
                    Image(systemName: "trash")
                    Text(String(format: NSLocalizedString("wishlist.deleteCount", comment: ""), viewModel.selectedItemIds.count))
                }
                .font(.system(size: 14, weight: .semibold))
                .foregroundColor(.white)
                .padding(.horizontal, OffriiTheme.spacingBase)
                .padding(.vertical, OffriiTheme.spacingSM)
                .background(viewModel.selectedItemIds.isEmpty ? OffriiTheme.textMuted : OffriiTheme.danger)
                .cornerRadius(OffriiTheme.cornerRadiusXL)
            }
            .disabled(viewModel.selectedItemIds.isEmpty)
        }
        .padding(.horizontal, OffriiTheme.spacingLG)
        .padding(.vertical, OffriiTheme.spacingSM)
        .background(.ultraThinMaterial)
        .transition(.move(edge: .bottom))
    }

    // MARK: - Helpers

    private var sortLabel: String {
        switch viewModel.sortField {
        case "priority": return NSLocalizedString("wishlist.sort.priority", comment: "")
        case "name":     return NSLocalizedString("wishlist.sort.name", comment: "")
        default:         return NSLocalizedString("wishlist.sort.date", comment: "")
        }
    }
}

extension UUID: @retroactive Identifiable {
    public var id: UUID { self }
}
