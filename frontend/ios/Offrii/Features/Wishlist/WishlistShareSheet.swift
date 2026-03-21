// swiftlint:disable file_length
import NukeUI
import SwiftUI

// MARK: - Share Tab

private enum ShareTab: String, CaseIterable {
    case proches
    case liens

    var label: String {
        switch self {
        case .proches: return NSLocalizedString("share.tab.proches", comment: "")
        case .liens: return NSLocalizedString("share.tab.liens", comment: "")
        }
    }
}

// MARK: - Share Scope

private enum ShareScope: String, CaseIterable {
    case all
    case category
    case selection
}

enum LinkTTL: String, CaseIterable {
    case oneDay
    case oneWeek
    case oneMonth
    case never

    var label: String {
        switch self {
        case .oneDay:      return NSLocalizedString("share.ttl.oneDay", comment: "")
        case .oneWeek:     return NSLocalizedString("share.ttl.oneWeek", comment: "")
        case .oneMonth:    return NSLocalizedString("share.ttl.oneMonth", comment: "")
        case .never:       return NSLocalizedString("share.ttl.never", comment: "")
        }
    }

    var expiresAt: String? {
        let calendar = Calendar.current
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime]
        switch self {
        case .oneDay: return formatter.string(from: calendar.date(byAdding: .day, value: 1, to: Date())!)
        case .oneWeek: return formatter.string(from: calendar.date(byAdding: .day, value: 7, to: Date())!)
        case .oneMonth: return formatter.string(from: calendar.date(byAdding: .month, value: 1, to: Date())!)
        case .never: return nil
        }
    }
}

// MARK: - WishlistShareSheet

// swiftlint:disable:next type_body_length
struct WishlistShareSheet: View {
    var items: [Item] = []
    var selectedItemIds: Set<UUID> = []
    var categories: [CategoryResponse] = []
    var privateItemCount: Int = 0

    @Environment(\.dismiss) private var dismiss

    // Tab
    @State private var selectedTab: ShareTab = .proches

    // Shared state (used by both tabs)
    @State private var scope: ShareScope = .all
    @State private var selectedCategoryIds: Set<UUID> = []
    @State private var pickedItemIds: Set<UUID> = []
    @State private var showItemPicker = false

    // Proches
    @State private var circles: [OffriiCircle] = []
    @State private var isLoadingCircles = false
    @State private var selectedCircleIds: Set<UUID> = []
    @State private var isSharing = false
    @State private var shareRules: [UUID: CircleShareRuleSummary] = [:]

    // Links
    @State private var shareLinks: [ShareLinkResponse] = []
    @State private var isLoadingLinks = false
    @State private var editingLink: ShareLinkResponse?
    @State private var permission: String = "view_and_claim"
    @State private var linkLabel: String = ""
    @State private var linkTTL: LinkTTL = .never
    @State private var isCreatingLink = false
    @State private var justCreatedLink: String?

    // Toasts
    @State private var toastMessage: String?
    @State private var linkToDelete: UUID?

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                Picker("", selection: $selectedTab) {
                    ForEach(ShareTab.allCases, id: \.rawValue) { tab in
                        Text(tab.label).tag(tab)
                    }
                }
                .pickerStyle(.segmented)
                .padding(.horizontal, OffriiTheme.spacingLG)
                .padding(.vertical, OffriiTheme.spacingSM)

                switch selectedTab {
                case .proches:
                    prochesContent
                case .liens:
                    liensContent
                }
            }
            .background(OffriiTheme.background)
            .navigationTitle(NSLocalizedString("share.title", comment: ""))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button { dismiss() } label: {
                        Image(systemName: "xmark")
                            .font(.system(size: 15, weight: .semibold))
                            .foregroundColor(OffriiTheme.textSecondary)
                    }
                }
            }
            .sheet(item: $editingLink) { link in
                EditShareLinkSheet(link: link) { updated in
                    if let idx = shareLinks.firstIndex(where: { $0.id == updated.id }) {
                        shareLinks[idx] = updated
                    }
                }
                .presentationDetents([.medium])
            }
            .sheet(isPresented: $showItemPicker) {
                ItemPickerSheet(items: items.filter { !$0.isPrivate }, selectedIds: $pickedItemIds)
                    .presentationDetents([.medium, .large])
            }
            .task {
                async let circlesLoad: () = loadCircles()
                async let linksLoad: () = loadShareLinks()
                _ = await (circlesLoad, linksLoad)
            }
            .alert(
                NSLocalizedString("share.deleteLink.title", comment: ""),
                isPresented: Binding(
                    get: { linkToDelete != nil },
                    set: { if !$0 { linkToDelete = nil } }
                )
            ) {
                Button(NSLocalizedString("common.delete", comment: ""), role: .destructive) {
                    if let id = linkToDelete {
                        Task { await deleteLink(id: id) }
                    }
                    linkToDelete = nil
                }
                Button(NSLocalizedString("common.cancel", comment: ""), role: .cancel) {
                    linkToDelete = nil
                }
            } message: {
                Text(NSLocalizedString("share.deleteLink.message", comment: ""))
            }
        }
    }

    // MARK: - Scope Selector (shared between tabs)

    private var scopeSelector: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("share.whatToShare", comment: ""))
                .font(OffriiTypography.subheadline)
                .foregroundColor(OffriiTheme.textMuted)

            // All
            scopeRadio(.all, icon: "list.bullet", label: NSLocalizedString("share.scopeAll", comment: ""))
            if scope == .all {
                dynamicHint
            }

            // Category (multi-select)
            scopeRadio(.category, icon: "tag.fill", label: NSLocalizedString("share.scopeCategories", comment: ""))
            if scope == .category {
                dynamicHint
            }

            if scope == .category && !categories.isEmpty {
                categoryChips
            }

            // Selection
            Button {
                withAnimation(OffriiAnimation.snappy) { scope = .selection }
                showItemPicker = true
            } label: {
                HStack(spacing: OffriiTheme.spacingSM) {
                    Image(systemName: scope == .selection ? "largecircle.fill.circle" : "circle")
                        .font(.system(size: 18))
                        .foregroundColor(scope == .selection ? OffriiTheme.primary : OffriiTheme.textMuted)
                    Image(systemName: "checkmark.circle")
                        .font(.system(size: 13))
                        .foregroundColor(OffriiTheme.textSecondary)
                    Text(NSLocalizedString("share.scopeSelection", comment: ""))
                        .font(OffriiTypography.body)
                        .foregroundColor(OffriiTheme.text)
                    Spacer()
                }
            }
            .buttonStyle(.plain)

            if scope == .selection && !pickedItemIds.isEmpty {
                pickedItemsPreview
            }
        }
        .padding(.horizontal, OffriiTheme.spacingLG)
    }

    private var categoryChips: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                ForEach(categories, id: \.id) { cat in
                    let isSelected = selectedCategoryIds.contains(cat.id)
                    OffriiChip(
                        title: cat.name,
                        isSelected: isSelected
                    ) {
                        if isSelected {
                            selectedCategoryIds.remove(cat.id)
                        } else {
                            selectedCategoryIds.insert(cat.id)
                        }
                    }
                }
            }
        }
        .padding(.leading, OffriiTheme.spacingXXL)
    }

    private var pickedItemsPreview: some View {
        VStack(alignment: .leading, spacing: 4) {
            ForEach(items.filter { pickedItemIds.contains($0.id) }) { item in
                HStack(spacing: 6) {
                    Image(systemName: "gift.fill")
                        .font(.system(size: 10))
                        .foregroundColor(OffriiTheme.primary)
                    Text(item.name)
                        .font(.system(size: 12))
                        .foregroundColor(OffriiTheme.text)
                        .lineLimit(1)
                }
            }
            Button { showItemPicker = true } label: {
                Text(NSLocalizedString("share.modifySelection", comment: ""))
                    .font(.system(size: 12, weight: .medium))
                    .foregroundColor(OffriiTheme.primary)
            }
        }
        .padding(.leading, OffriiTheme.spacingXXL)
    }

    private func scopeRadio(_ option: ShareScope, icon: String, label: String) -> some View {
        Button {
            withAnimation(OffriiAnimation.snappy) { scope = option }
        } label: {
            HStack(spacing: OffriiTheme.spacingSM) {
                Image(systemName: scope == option ? "largecircle.fill.circle" : "circle")
                    .font(.system(size: 18))
                    .foregroundColor(scope == option ? OffriiTheme.primary : OffriiTheme.textMuted)
                Image(systemName: icon)
                    .font(.system(size: 13))
                    .foregroundColor(OffriiTheme.textSecondary)
                Text(label)
                    .font(OffriiTypography.body)
                    .foregroundColor(OffriiTheme.text)
                Spacer()
            }
        }
        .buttonStyle(.plain)
    }

    private var dynamicHint: some View {
        HStack(spacing: 4) {
            Image(systemName: "arrow.triangle.2.circlepath")
                .font(.system(size: 10))
            Text(NSLocalizedString("shareRule.dynamicHint", comment: ""))
                .font(OffriiTypography.caption)
        }
        .foregroundColor(OffriiTheme.textMuted)
        .padding(.leading, 34)
    }

    /// Resolve item IDs based on current scope.
    private var resolvedItemIds: [UUID] {
        switch scope {
        case .all:
            return items.filter { $0.isActive && !$0.isPrivate }.map(\.id)
        case .category:
            return items.filter { item in
                guard let catId = item.categoryId else { return false }
                return selectedCategoryIds.contains(catId) && !item.isPrivate
            }.map(\.id)
        case .selection:
            return Array(pickedItemIds)
        }
    }

    private var isScopeEmpty: Bool {
        switch scope {
        case .all: return false
        case .category: return selectedCategoryIds.isEmpty
        case .selection: return pickedItemIds.isEmpty
        }
    }

    // MARK: - Onglet Proches

    private var prochesContent: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
                // Scope selector
                scopeSelector

                Divider().padding(.horizontal, OffriiTheme.spacingLG)

                // Circle multi-select
                circleSelector

                // Share button
                if !selectedCircleIds.isEmpty {
                    shareToCirclesButton
                }
            }
            .padding(.vertical, OffriiTheme.spacingBase)
        }
    }

    private var circleSelector: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("share.toWhom", comment: ""))
                .font(OffriiTypography.subheadline)
                .foregroundColor(OffriiTheme.textMuted)
                .padding(.horizontal, OffriiTheme.spacingLG)

            if isLoadingCircles {
                SkeletonList(count: 3)
                    .padding(.horizontal, OffriiTheme.spacingLG)
            } else if circles.isEmpty {
                OffriiEmptyState(
                    icon: "person.2.fill",
                    title: NSLocalizedString("share.noCircles", comment: ""),
                    subtitle: NSLocalizedString("share.noCirclesSubtitle", comment: "")
                )
            } else {
                VStack(spacing: OffriiTheme.spacingXS) {
                    ForEach(circles) { circle in
                        circleCheckRow(circle)
                    }
                }
                .padding(.horizontal, OffriiTheme.spacingLG)
            }
        }
    }

    @ViewBuilder
    private func circleShareStatus(_ circle: OffriiCircle) -> some View {
        let sharedCount = items.filter { $0.sharedCircles.contains(where: { $0.id == circle.id }) }.count
        let totalActive = items.filter { $0.isActive && !$0.isPrivate }.count
        let ruleMode = shareRules[circle.id]?.shareMode

        if ruleMode == "all" {
            Label(NSLocalizedString("share.ruleAll", comment: ""), systemImage: "checkmark.circle.fill")
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.primary)
        } else if ruleMode == "categories", let catCount = shareRules[circle.id]?.categoryCount {
            Label(
                String(format: NSLocalizedString("share.ruleCategories", comment: ""), catCount),
                systemImage: "folder.fill"
            )
            .font(OffriiTypography.caption)
            .foregroundColor(OffriiTheme.primary)
        } else if sharedCount == 0 {
            Text(NSLocalizedString("share.noSharedItems", comment: ""))
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.textMuted)
        } else if sharedCount == totalActive {
            Text(NSLocalizedString("share.seesAllList", comment: ""))
                .font(OffriiTypography.caption).fontWeight(.medium)
                .foregroundColor(OffriiTheme.primary)
        } else {
            Text(sharedCount == 1
                ? NSLocalizedString("share.sharedItemSingular", comment: "")
                : String(format: NSLocalizedString("share.sharedItemPlural", comment: ""), sharedCount))
                .font(OffriiTypography.caption).fontWeight(.medium)
                .foregroundColor(OffriiTheme.primary)
        }
    }

    private func circleCheckRow(_ circle: OffriiCircle) -> some View {
        let isSelected = selectedCircleIds.contains(circle.id)
        let isRuleAll = shareRules[circle.id]?.shareMode == "all"

        return Button {
            withAnimation(OffriiAnimation.snappy) {
                if isSelected {
                    selectedCircleIds.remove(circle.id)
                } else {
                    selectedCircleIds.insert(circle.id)
                }
            }
        } label: {
            HStack(spacing: OffriiTheme.spacingMD) {
                circleAvatar(circle)

                VStack(alignment: .leading, spacing: 2) {
                    Text(circle.name ?? NSLocalizedString("circles.unnamed", comment: ""))
                        .font(OffriiTypography.body)
                        .foregroundColor(OffriiTheme.text)

                    circleShareStatus(circle)
                }

                Spacer()

                if isRuleAll {
                    Image(systemName: "checkmark.circle.fill")
                        .font(.system(size: 22))
                        .foregroundColor(OffriiTheme.primary)
                } else {
                    Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                        .font(.system(size: 22))
                        .foregroundColor(isSelected ? OffriiTheme.primary : OffriiTheme.textMuted)
                }
            }
            .padding(OffriiTheme.spacingBase)
            .background(isSelected ? OffriiTheme.primary.opacity(0.05) : OffriiTheme.card)
            .cornerRadius(OffriiTheme.cornerRadiusLG)
            .opacity(isRuleAll ? 0.7 : 1.0)
        }
        .buttonStyle(.plain)
        .disabled(isRuleAll)
        .animation(OffriiAnimation.snappy, value: isSelected)
    }

    private var shareToCirclesButton: some View {
        VStack(spacing: OffriiTheme.spacingSM) {
            if let msg = toastMessage {
                toastView(msg)
            }

            OffriiButton(
                String(format: NSLocalizedString("share.shareToCount", comment: ""), selectedCircleIds.count),
                isLoading: isSharing,
                isDisabled: isScopeEmpty
            ) {
                Task { await shareToSelectedCircles() }
            }
            .padding(.horizontal, OffriiTheme.spacingLG)
        }
    }

    @ViewBuilder
    private func circleAvatar(_ circle: OffriiCircle) -> some View {
        if let imageUrl = circle.imageUrl, let url = URL(string: imageUrl) {
            LazyImage(url: url) { state in
                if let image = state.image {
                    image.resizable().aspectRatio(contentMode: .fill)
                        .frame(width: 40, height: 40).clipShape(Circle())
                } else {
                    AvatarView(circle.name, size: .medium)
                }
            }
        } else {
            AvatarView(
                circle.isDirect ? circle.memberNames.first ?? circle.name : circle.name,
                size: .medium
            )
        }
    }

    // MARK: - Onglet Liens

    private var liensContent: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {
                // Section 1: Create link
                createLinkSection

                Divider().padding(.horizontal, OffriiTheme.spacingLG)

                // Section 2: Existing links
                activeLinksSection
            }
            .padding(.vertical, OffriiTheme.spacingBase)
        }
    }

    // MARK: - Create Link Section

    private var createLinkSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("share.createLink", comment: ""))
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)
                .padding(.horizontal, OffriiTheme.spacingLG)

            scopeSelector
            linkOptionsSection

            if justCreatedLink != nil { linkCreatedToast }

            createLinkButton
        }
    }

    private var linkOptionsSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            // Permissions
            VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                Text(NSLocalizedString("share.permissions", comment: ""))
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textMuted)
                Picker("", selection: $permission) {
                    Text(NSLocalizedString("share.permViewAndClaim", comment: "")).tag("view_and_claim")
                    Text(NSLocalizedString("share.permViewOnly", comment: "")).tag("view_only")
                }
                .pickerStyle(.segmented)
            }

            // Link name
            VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                Text(NSLocalizedString("share.linkName", comment: ""))
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textMuted)
                TextField(NSLocalizedString("share.linkNamePlaceholder", comment: ""), text: $linkLabel)
                    .font(OffriiTypography.body)
                    .padding(.horizontal, OffriiTheme.spacingMD)
                    .padding(.vertical, 10)
                    .background(OffriiTheme.surface)
                    .cornerRadius(OffriiTheme.cornerRadiusSM)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusSM)
                            .strokeBorder(OffriiTheme.border, lineWidth: 1)
                    )
            }

            // TTL
            VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                Text(NSLocalizedString("share.ttl", comment: ""))
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textMuted)
                Picker("", selection: $linkTTL) {
                    ForEach(LinkTTL.allCases, id: \.rawValue) { ttl in
                        Text(ttl.label).tag(ttl)
                    }
                }
                .pickerStyle(.segmented)
            }

            // Private items warning
            if privateItemCount > 0 && scope == .all {
                HStack(spacing: 4) {
                    Image(systemName: "lock.fill").font(.system(size: 10))
                    Text(String(format: NSLocalizedString("share.privateExcluded", comment: ""), privateItemCount))
                        .font(OffriiTypography.caption)
                }
                .foregroundColor(OffriiTheme.textMuted)
            }
        }
        .padding(.horizontal, OffriiTheme.spacingLG)
    }

    private var linkCreatedToast: some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            Image(systemName: "checkmark.circle.fill").foregroundColor(OffriiTheme.primary)
            Text(NSLocalizedString("share.linkCopied", comment: ""))
                .font(OffriiTypography.subheadline).foregroundColor(OffriiTheme.primary)
        }
        .frame(maxWidth: .infinity)
        .padding(OffriiTheme.spacingSM)
        .background(OffriiTheme.primary.opacity(0.1))
        .cornerRadius(OffriiTheme.cornerRadiusSM)
        .padding(.horizontal, OffriiTheme.spacingLG)
        .transition(.move(edge: .top).combined(with: .opacity))
        .onAppear {
            DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
                withAnimation { justCreatedLink = nil }
            }
        }
    }

    private var createLinkButton: some View {
        Button {
            Task { await createLink() }
        } label: {
            HStack(spacing: OffriiTheme.spacingSM) {
                Image(systemName: "link.badge.plus")
                Text(NSLocalizedString("share.createLink", comment: ""))
            }
            .font(.system(size: 15, weight: .semibold))
            .foregroundColor(.white)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 14)
            .background(isScopeEmpty ? OffriiTheme.textMuted : OffriiTheme.primary)
            .cornerRadius(OffriiTheme.cornerRadiusMD)
        }
        .disabled(isScopeEmpty || isCreatingLink)
        .overlay { if isCreatingLink { ProgressView().tint(.white) } }
        .padding(.horizontal, OffriiTheme.spacingLG)
    }

    // MARK: - Active Links

    private var activeLinksSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            if isLoadingLinks {
                SkeletonList(count: 2)
                    .padding(.horizontal, OffriiTheme.spacingLG)
            } else if shareLinks.isEmpty {
                VStack(spacing: OffriiTheme.spacingSM) {
                    Spacer().frame(height: 20)
                    OffriiEmptyState(
                        icon: "link",
                        title: NSLocalizedString("share.noLinks", comment: ""),
                        subtitle: NSLocalizedString("share.noLinksSubtitle", comment: "")
                    )
                }
            } else {
                Text(NSLocalizedString("share.activeLinks", comment: "") + " (\(shareLinks.count))")
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)
                    .padding(.horizontal, OffriiTheme.spacingLG)

                ForEach(shareLinks) { link in
                    linkCard(link)
                }
            }
        }
    }

    private func linkCard(_ link: ShareLinkResponse) -> some View {
        let isDisabled = link.isActive == false
        return VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            linkCardHeader(link, isDisabled: isDisabled)
            linkCardUrl(link, isDisabled: isDisabled)
            linkCardInfo(link)
            linkCardActions(link)
        }
        .padding(OffriiTheme.spacingMD)
        .background(isDisabled ? OffriiTheme.surface.opacity(0.6) : OffriiTheme.surface)
        .cornerRadius(OffriiTheme.cornerRadiusMD)
        .overlay(
            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                .strokeBorder(isDisabled ? OffriiTheme.border : .clear, lineWidth: 1)
        )
        .contextMenu { linkContextMenu(link) }
        .padding(.horizontal, OffriiTheme.spacingLG)
    }

    @ViewBuilder
    private func linkCardHeader(_ link: ShareLinkResponse, isDisabled: Bool) -> some View {
        HStack(spacing: OffriiTheme.spacingSM) {
            if isDisabled {
                Text(NSLocalizedString("share.linkDisabled", comment: ""))
                    .font(.system(size: 10, weight: .semibold)).foregroundColor(.white)
                    .padding(.horizontal, 6).padding(.vertical, 2)
                    .background(OffriiTheme.textMuted).cornerRadius(4)
            }
            if let label = link.label, !label.isEmpty {
                Text(label).font(.system(size: 13, weight: .semibold))
                    .foregroundColor(isDisabled ? OffriiTheme.textMuted : OffriiTheme.text)
            }
        }
    }

    @ViewBuilder
    private func linkCardUrl(_ link: ShareLinkResponse, isDisabled: Bool) -> some View {
        if !isDisabled, let url = URL(string: link.displayUrl) {
            Link(destination: url) {
                HStack(spacing: 6) {
                    Image(systemName: "link").font(.system(size: 11))
                    Text(link.displayUrl).font(.system(size: 12, weight: .medium)).lineLimit(1)
                    Spacer()
                    Image(systemName: "arrow.up.right").font(.system(size: 10))
                }
                .foregroundColor(OffriiTheme.primary)
            }
        } else {
            HStack(spacing: 6) {
                Image(systemName: "link").font(.system(size: 11))
                Text(link.displayUrl).font(.system(size: 12, weight: .medium)).lineLimit(1).strikethrough(isDisabled)
            }
            .foregroundColor(OffriiTheme.textMuted)
        }
    }

    private func linkCardInfo(_ link: ShareLinkResponse) -> some View {
        HStack(spacing: 4) {
            Text(scopeLabel(link.scope)); Text("\u{00B7}")
            Text(permissionLabel(link.permissions)); Text("\u{00B7}")
            if let exp = link.expiresAt {
                Text(exp, style: .relative)
            } else {
                Text(NSLocalizedString("share.ttl.never", comment: ""))
            }
        }
        .font(.system(size: 10)).foregroundColor(OffriiTheme.textMuted)
    }

    private func linkCardActions(_ link: ShareLinkResponse) -> some View {
        HStack(spacing: OffriiTheme.spacingBase) {
            Spacer()
            Button {
                UIPasteboard.general.string = link.displayUrl
                OffriiHaptics.success()
                showToast(NSLocalizedString("share.linkCopied", comment: ""))
            } label: {
                Label(NSLocalizedString("share.copyLink", comment: ""), systemImage: "doc.on.doc")
                    .font(.system(size: 11, weight: .medium)).foregroundColor(OffriiTheme.primary)
            }
            if let shareUrl = URL(string: link.displayUrl) {
                ShareLink(item: shareUrl) {
                    Label(NSLocalizedString("share.sendDirect", comment: ""), systemImage: "square.and.arrow.up")
                        .font(.system(size: 11, weight: .medium)).foregroundColor(OffriiTheme.primary)
                }
            }
            Button { linkToDelete = link.id } label: {
                Label(NSLocalizedString("common.delete", comment: ""), systemImage: "trash")
                    .font(.system(size: 11, weight: .medium)).foregroundColor(OffriiTheme.danger)
            }
        }
    }

    @ViewBuilder
    private func linkContextMenu(_ link: ShareLinkResponse) -> some View {
        Button { editingLink = link } label: {
            Label(NSLocalizedString("wishlist.edit", comment: ""), systemImage: "pencil")
        }
        Button {
            UIPasteboard.general.string = link.displayUrl; OffriiHaptics.success()
            showToast(NSLocalizedString("share.linkCopied", comment: ""))
        } label: {
            Label(NSLocalizedString("share.copyLink", comment: ""), systemImage: "doc.on.doc")
        }
        Divider()
        Button(role: .destructive) { linkToDelete = link.id } label: {
            Label(NSLocalizedString("common.delete", comment: ""), systemImage: "trash")
        }
    }

    // MARK: - Toast

    private func toastView(_ message: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: "checkmark.circle.fill")
            Text(message).font(.system(size: 13, weight: .medium))
        }
        .foregroundColor(OffriiTheme.primary)
        .frame(maxWidth: .infinity)
        .padding(OffriiTheme.spacingSM)
        .background(OffriiTheme.primary.opacity(0.1))
        .cornerRadius(OffriiTheme.cornerRadiusSM)
        .padding(.horizontal, OffriiTheme.spacingLG)
        .transition(.move(edge: .top).combined(with: .opacity))
        .onAppear {
            DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                withAnimation { toastMessage = nil }
            }
        }
    }

    // MARK: - Helpers

    private func scopeLabel(_ scope: String?) -> String {
        switch scope {
        case "all": return NSLocalizedString("share.scopeAllLabel", comment: "")
        case "category": return NSLocalizedString("share.scopeCategory", comment: "")
        case "selection": return NSLocalizedString("share.scopeSelection", comment: "")
        default: return NSLocalizedString("share.scopeAllLabel", comment: "")
        }
    }

    private func permissionLabel(_ perm: String?) -> String {
        switch perm {
        case "view_only": return NSLocalizedString("share.permViewOnly", comment: "")
        default: return NSLocalizedString("share.permViewAndClaim", comment: "")
        }
    }

    // MARK: - Actions

    private func loadCircles() async {
        isLoadingCircles = true
        do {
            async let circlesTask = CircleService.shared.listCircles()
            async let rulesTask = CircleService.shared.listMyShareRules()
            circles = try await circlesTask
            let rules = try await rulesTask
            shareRules = Dictionary(uniqueKeysWithValues: rules.map { ($0.circleId, $0) })
        } catch { /* Best-effort refresh */ }
        isLoadingCircles = false
    }

    private func loadShareLinks() async {
        isLoadingLinks = true
        do {
            let response: PaginatedResponse<ShareLinkResponse> = try await APIClient.shared.request(.listShareLinks)
            shareLinks = response.data
        } catch { /* Best-effort refresh */ }
        isLoadingLinks = false
    }

    private func shareToSelectedCircles() async {
        isSharing = true

        for circleId in selectedCircleIds {
            do {
                switch scope {
                case .all:
                    try await CircleService.shared.setShareRule(
                        circleId: circleId, mode: "all"
                    )
                case .category:
                    try await CircleService.shared.setShareRule(
                        circleId: circleId, mode: "categories",
                        categoryIds: Array(selectedCategoryIds)
                    )
                case .selection:
                    try await CircleService.shared.setShareRule(
                        circleId: circleId, mode: "selection"
                    )
                    let itemIds = Array(pickedItemIds)
                    if !itemIds.isEmpty {
                        try await CircleService.shared.batchShareItems(
                            circleId: circleId, itemIds: itemIds
                        )
                    }
                }
            } catch {
                showToast(NSLocalizedString("error.serverError", comment: ""))
            }
        }

        // Refresh share rules state
        if let rules = try? await CircleService.shared.listMyShareRules() {
            shareRules = Dictionary(uniqueKeysWithValues: rules.map { ($0.circleId, $0) })
        }

        OffriiHaptics.success()
        showToast(NSLocalizedString("share.sharedSuccess", comment: ""))
        selectedCircleIds.removeAll()
        isSharing = false
    }

    private func createLink() async {
        isCreatingLink = true

        let scopeStr: String
        let scopeData: ScopeData?

        switch scope {
        case .all:
            scopeStr = "all"
            scopeData = nil
        case .category:
            let itemIds = resolvedItemIds
            scopeStr = "selection"
            scopeData = ScopeData(categoryId: nil, itemIds: itemIds.map { $0.uuidString })
        case .selection:
            scopeStr = "selection"
            scopeData = ScopeData(categoryId: nil, itemIds: pickedItemIds.map { $0.uuidString })
        }

        let body = CreateShareLinkBody(
            expiresAt: linkTTL.expiresAt,
            label: linkLabel.trimmingCharacters(in: .whitespaces).isEmpty
                ? nil : linkLabel.trimmingCharacters(in: .whitespaces),
            permissions: permission,
            scope: scopeStr,
            scopeData: scopeData
        )

        do {
            let response: ShareLinkResponse = try await APIClient.shared.request(.createShareLink(body))
            UIPasteboard.general.string = response.displayUrl

            withAnimation(OffriiAnimation.defaultSpring) {
                justCreatedLink = response.displayUrl
                shareLinks.insert(response, at: 0)
            }
            OffriiHaptics.success()
        } catch {
            OffriiHaptics.error()
        }
        isCreatingLink = false
    }

    private func deleteLink(id: UUID) async {
        do {
            try await APIClient.shared.requestVoid(.deleteShareLink(id: id))
            withAnimation { shareLinks.removeAll { $0.id == id } }
            OffriiHaptics.success()
            showToast(NSLocalizedString("share.linkDeleted", comment: ""))
        } catch {
            OffriiHaptics.error()
        }
    }

    private func showToast(_ message: String) {
        withAnimation(OffriiAnimation.defaultSpring) { toastMessage = message }
    }
}
