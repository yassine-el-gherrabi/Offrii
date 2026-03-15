import SwiftUI

// MARK: - Share Scope

private enum ShareScope: String, CaseIterable {
    case all
    case category
    case selection
}

private enum LinkTTL: String, CaseIterable {
    case never
    case oneWeek
    case oneMonth
    case threeMonths

    var label: String {
        switch self {
        case .never:       return NSLocalizedString("share.ttl.never", comment: "")
        case .oneWeek:     return NSLocalizedString("share.ttl.oneWeek", comment: "")
        case .oneMonth:    return NSLocalizedString("share.ttl.oneMonth", comment: "")
        case .threeMonths: return NSLocalizedString("share.ttl.threeMonths", comment: "")
        }
    }

    var expiresAt: String? {
        let calendar = Calendar.current
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime]
        switch self {
        case .never: return nil
        case .oneWeek: return formatter.string(from: calendar.date(byAdding: .day, value: 7, to: Date())!)
        case .oneMonth: return formatter.string(from: calendar.date(byAdding: .month, value: 1, to: Date())!)
        case .threeMonths: return formatter.string(from: calendar.date(byAdding: .month, value: 3, to: Date())!)
        }
    }
}

// MARK: - WishlistShareSheet

struct WishlistShareSheet: View {
    var items: [Item] = []
    var selectedItemIds: Set<UUID> = []
    var categories: [CategoryResponse] = []
    var privateItemCount: Int = 0

    @Environment(\.dismiss) private var dismiss

    // Circles
    @State private var circles: [OffriiCircle] = []
    @State private var isLoadingCircles = false
    @State private var sharedCircleIds: Set<UUID> = []

    // Link creation
    @State private var scope: ShareScope = .all
    @State private var selectedCategoryIds: Set<UUID> = []
    @State private var pickedItemIds: Set<UUID> = []
    @State private var showItemPicker = false
    @State private var permission: String = "view_and_claim"
    @State private var linkLabel: String = ""
    @State private var linkTTL: LinkTTL = .never
    @State private var showAllItemsDetail = false
    @State private var isCreatingLink = false
    @State private var justCreatedLink: String?

    // Active links
    @State private var shareLinks: [ShareLinkResponse] = []
    @State private var isLoadingLinks = false
    @State private var toastMessage: String?
    /// Maps link ID → item names for links created in this session
    @State private var linkItemNames: [UUID: [String]] = [:]
    @State private var editingLink: ShareLinkResponse?

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: OffriiTheme.spacingLG) {

                    // Header description
                    Text(NSLocalizedString("share.description", comment: ""))
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.textSecondary)
                        .padding(.horizontal, OffriiTheme.spacingLG)

                    // ── SECTION 1: Cercles ──
                    circlesSection

                    Divider().padding(.horizontal, OffriiTheme.spacingLG)

                    // ── SECTION 2: Créer un lien ──
                    createLinkSection

                    Divider().padding(.horizontal, OffriiTheme.spacingLG)

                    // ── SECTION 3: Liens actifs ──
                    activeLinksSection
                }
                .padding(.vertical, OffriiTheme.spacingBase)
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
                async let c: () = loadCircles()
                async let l: () = loadShareLinks()
                _ = await (c, l)
            }
        }
    }

    // MARK: - Section 1: Cercles

    private var circlesSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("share.yourCircles", comment: ""))
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)
                .padding(.horizontal, OffriiTheme.spacingLG)

            if isLoadingCircles {
                SkeletonRow()
                    .padding(.horizontal, OffriiTheme.spacingLG)
            } else if circles.isEmpty {
                Text(NSLocalizedString("share.noCircles", comment: ""))
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textMuted)
                    .padding(.horizontal, OffriiTheme.spacingLG)
            } else {
                ForEach(circles) { circle in
                    let isShared = sharedCircleIds.contains(circle.id)

                    HStack(spacing: OffriiTheme.spacingMD) {
                        Image(systemName: circle.isDirect ? "bubble.left.fill" : "person.2.fill")
                            .font(.system(size: 14))
                            .foregroundColor(OffriiTheme.primary)
                            .frame(width: 32, height: 32)
                            .background(OffriiTheme.primary.opacity(0.1))
                            .clipShape(Circle())

                        VStack(alignment: .leading, spacing: 1) {
                            Text(circle.name ?? NSLocalizedString("circles.unnamed", comment: ""))
                                .font(OffriiTypography.body)
                                .foregroundColor(OffriiTheme.text)
                            Text(String(format: NSLocalizedString("circles.memberCount", comment: ""), circle.memberCount))
                                .font(OffriiTypography.caption)
                                .foregroundColor(OffriiTheme.textMuted)
                        }

                        Spacer()

                        if isShared {
                            HStack(spacing: 4) {
                                Image(systemName: "checkmark")
                                    .font(.system(size: 11, weight: .bold))
                                Text(NSLocalizedString("share.sharedSuccess", comment: ""))
                                    .font(.system(size: 12, weight: .medium))
                            }
                            .foregroundColor(OffriiTheme.success)
                        } else {
                            Button {
                                Task { await shareToCircle(circle) }
                            } label: {
                                Text(NSLocalizedString("share.addPeople", comment: ""))
                                    .font(.system(size: 12, weight: .semibold))
                                    .foregroundColor(.white)
                                    .padding(.horizontal, 12)
                                    .padding(.vertical, 6)
                                    .background(OffriiTheme.primary)
                                    .cornerRadius(OffriiTheme.cornerRadiusXL)
                            }
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                    .padding(.vertical, OffriiTheme.spacingXS)
                    .animation(OffriiAnimation.snappy, value: isShared)
                }
            }
        }
    }

    // MARK: - Section 2: Créer un lien

    private var createLinkSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            Text(NSLocalizedString("share.createLink", comment: ""))
                .font(OffriiTypography.headline)
                .foregroundColor(OffriiTheme.text)
                .padding(.horizontal, OffriiTheme.spacingLG)

            // Scope picker
            VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                Text(NSLocalizedString("share.whatToShare", comment: ""))
                    .font(OffriiTypography.subheadline)
                    .foregroundColor(OffriiTheme.textMuted)

                // Radio: All + expandable detail
                HStack {
                    scopeOption(.all, icon: "list.bullet", label: NSLocalizedString("share.scopeAll", comment: ""))

                    if scope == .all {
                        let activeNonPrivate = items.filter { $0.isActive && !$0.isPrivate }
                        Text("(\(activeNonPrivate.count))")
                            .font(.system(size: 12))
                            .foregroundColor(OffriiTheme.textMuted)

                        Button {
                            withAnimation(OffriiAnimation.snappy) {
                                showAllItemsDetail.toggle()
                            }
                        } label: {
                            Image(systemName: showAllItemsDetail ? "chevron.up" : "chevron.down")
                                .font(.system(size: 10, weight: .semibold))
                                .foregroundColor(OffriiTheme.textMuted)
                        }
                    }
                }

                if scope == .all && showAllItemsDetail {
                    VStack(alignment: .leading, spacing: 3) {
                        ForEach(items.filter { $0.isActive && !$0.isPrivate }) { item in
                            HStack(spacing: 6) {
                                Image(systemName: "gift.fill")
                                    .font(.system(size: 9))
                                    .foregroundColor(OffriiTheme.primary.opacity(0.6))
                                Text(item.name)
                                    .font(.system(size: 11))
                                    .foregroundColor(OffriiTheme.textSecondary)
                                    .lineLimit(1)
                            }
                        }
                    }
                    .padding(.leading, OffriiTheme.spacingXXL)
                    .transition(.opacity.combined(with: .move(edge: .top)))
                }

                // Radio: Categories (multi-select)
                scopeOption(.category, icon: "tag.fill", label: NSLocalizedString("share.scopeCategory", comment: ""))

                if scope == .category && !categories.isEmpty {
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: OffriiTheme.spacingSM) {
                            ForEach(categories, id: \.id) { cat in
                                let isSelected = selectedCategoryIds.contains(cat.id)
                                let style = CategoryStyle(icon: cat.icon)

                                Button {
                                    if isSelected {
                                        selectedCategoryIds.remove(cat.id)
                                    } else {
                                        selectedCategoryIds.insert(cat.id)
                                    }
                                } label: {
                                    HStack(spacing: 4) {
                                        Image(systemName: style.sfSymbol)
                                            .font(.system(size: 10))
                                        Text(cat.name)
                                            .font(.system(size: 12, weight: isSelected ? .semibold : .regular))
                                    }
                                    .foregroundColor(isSelected ? .white : OffriiTheme.textSecondary)
                                    .padding(.horizontal, 10)
                                    .padding(.vertical, 6)
                                    .background(isSelected ? style.chipColor : OffriiTheme.surface)
                                    .cornerRadius(OffriiTheme.cornerRadiusXL)
                                    .overlay(
                                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusXL)
                                            .strokeBorder(isSelected ? .clear : OffriiTheme.border, lineWidth: 1)
                                    )
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }
                    .padding(.leading, OffriiTheme.spacingXXL)
                }

                // Radio: Selection (pick specific items) — opens picker immediately
                Button {
                    withAnimation(OffriiAnimation.snappy) {
                        scope = .selection
                    }
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

                // Show picked items preview OR pick button
                if scope == .selection {
                    if pickedItemIds.isEmpty {
                        // Nothing selected yet — show pick button
                        Button {
                            showItemPicker = true
                        } label: {
                            HStack(spacing: 6) {
                                Image(systemName: "plus.circle")
                                    .font(.system(size: 14))
                                Text(NSLocalizedString("share.pickItems", comment: ""))
                                    .font(.system(size: 13, weight: .medium))
                            }
                            .foregroundColor(OffriiTheme.primary)
                        }
                        .padding(.leading, OffriiTheme.spacingXXL)
                    } else {
                        // Show selected items
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

                            Button {
                                showItemPicker = true
                            } label: {
                                Text(NSLocalizedString("share.modifySelection", comment: ""))
                                    .font(.system(size: 12, weight: .medium))
                                    .foregroundColor(OffriiTheme.primary)
                            }
                        }
                        .padding(.leading, OffriiTheme.spacingXXL)
                    }
                }
            }
            .padding(.horizontal, OffriiTheme.spacingLG)

            // Permissions picker
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
            .padding(.horizontal, OffriiTheme.spacingLG)

            // Link name (optional)
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
            .padding(.horizontal, OffriiTheme.spacingLG)

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
            .padding(.horizontal, OffriiTheme.spacingLG)

            // Private items warning
            if privateItemCount > 0 && scope == .all {
                HStack(spacing: 4) {
                    Image(systemName: "lock.fill")
                        .font(.system(size: 10))
                    Text(String(format: NSLocalizedString("share.privateExcluded", comment: ""), privateItemCount))
                        .font(OffriiTypography.caption)
                }
                .foregroundColor(OffriiTheme.textMuted)
                .padding(.horizontal, OffriiTheme.spacingLG)
            }

            // Success toast
            if justCreatedLink != nil {
                HStack(spacing: OffriiTheme.spacingSM) {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(OffriiTheme.success)
                    Text(NSLocalizedString("share.linkCopied", comment: ""))
                        .font(OffriiTypography.subheadline)
                        .foregroundColor(OffriiTheme.success)
                }
                .frame(maxWidth: .infinity)
                .padding(OffriiTheme.spacingSM)
                .background(OffriiTheme.success.opacity(0.1))
                .cornerRadius(OffriiTheme.cornerRadiusSM)
                .padding(.horizontal, OffriiTheme.spacingLG)
                .transition(.move(edge: .top).combined(with: .opacity))
                .onAppear {
                    DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
                        withAnimation { justCreatedLink = nil }
                    }
                }
            }

            // Action buttons
            VStack(spacing: OffriiTheme.spacingSM) {
                // Create + copy
                Button {
                    Task { await createLink(openShareSheet: false) }
                } label: {
                    HStack(spacing: OffriiTheme.spacingSM) {
                        Image(systemName: "doc.on.doc")
                        Text(NSLocalizedString("share.createAndCopy", comment: ""))
                    }
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundColor(.white)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 14)
                    .background(isCreateDisabled ? OffriiTheme.textMuted : OffriiTheme.primary)
                    .cornerRadius(OffriiTheme.cornerRadiusMD)
                }
                .disabled(isCreateDisabled || isCreatingLink)
                .overlay { if isCreatingLink { ProgressView().tint(.white) } }

                // Send directly via iOS share sheet
                Button {
                    Task { await createLink(openShareSheet: true) }
                } label: {
                    HStack(spacing: OffriiTheme.spacingSM) {
                        Image(systemName: "square.and.arrow.up")
                        Text(NSLocalizedString("share.sendDirect", comment: ""))
                    }
                    .font(.system(size: 15, weight: .medium))
                    .foregroundColor(OffriiTheme.primary)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 14)
                    .background(OffriiTheme.primary.opacity(0.1))
                    .cornerRadius(OffriiTheme.cornerRadiusMD)
                }
                .disabled(isCreateDisabled || isCreatingLink)
            }
            .padding(.horizontal, OffriiTheme.spacingLG)
        }
    }

    // MARK: - Section 3: Liens actifs

    private var activeLinksSection: some View {
        VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
            // Toast
            if let msg = toastMessage {
                HStack(spacing: 6) {
                    Image(systemName: "checkmark.circle.fill")
                    Text(msg)
                        .font(.system(size: 13, weight: .medium))
                }
                .foregroundColor(OffriiTheme.success)
                .frame(maxWidth: .infinity)
                .padding(OffriiTheme.spacingSM)
                .background(OffriiTheme.success.opacity(0.1))
                .cornerRadius(OffriiTheme.cornerRadiusSM)
                .padding(.horizontal, OffriiTheme.spacingLG)
                .transition(.move(edge: .top).combined(with: .opacity))
                .onAppear {
                    DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                        withAnimation { toastMessage = nil }
                    }
                }
            }

            if !shareLinks.isEmpty {
                Text(NSLocalizedString("share.activeLinks", comment: "") + " (\(shareLinks.count))")
                    .font(OffriiTypography.headline)
                    .foregroundColor(OffriiTheme.text)
                    .padding(.horizontal, OffriiTheme.spacingLG)

                ForEach(shareLinks) { link in
                    let isDisabled = link.isActive == false

                    VStack(alignment: .leading, spacing: OffriiTheme.spacingSM) {
                        // Disabled badge + label
                        HStack(spacing: OffriiTheme.spacingSM) {
                            if isDisabled {
                                Text(NSLocalizedString("share.linkDisabled", comment: ""))
                                    .font(.system(size: 10, weight: .semibold))
                                    .foregroundColor(.white)
                                    .padding(.horizontal, 6)
                                    .padding(.vertical, 2)
                                    .background(OffriiTheme.textMuted)
                                    .cornerRadius(4)
                            }

                            if let label = link.label, !label.isEmpty {
                                Text(label)
                                    .font(.system(size: 13, weight: .semibold))
                                    .foregroundColor(isDisabled ? OffriiTheme.textMuted : OffriiTheme.text)
                            }
                        }

                        // Tappable URL → opens default browser (disabled if inactive)
                        if !isDisabled, let url = URL(string: link.displayUrl) {
                            Link(destination: url) {
                                HStack(spacing: 6) {
                                    Image(systemName: "link")
                                        .font(.system(size: 11))
                                    Text(link.displayUrl)
                                        .font(.system(size: 12, weight: .medium))
                                        .lineLimit(1)
                                    Spacer()
                                    Image(systemName: "arrow.up.right")
                                        .font(.system(size: 10))
                                }
                                .foregroundColor(OffriiTheme.primary)
                            }
                        } else {
                            HStack(spacing: 6) {
                                Image(systemName: "link")
                                    .font(.system(size: 11))
                                Text(link.displayUrl)
                                    .font(.system(size: 12, weight: .medium))
                                    .lineLimit(1)
                                    .strikethrough(isDisabled)
                            }
                            .foregroundColor(OffriiTheme.textMuted)
                        }

                        // Info rows
                        VStack(alignment: .leading, spacing: 3) {
                            // Scope + permissions
                            HStack(spacing: 4) {
                                Text(scopeLabel(link.scope))
                                Text("·")
                                Text(permissionLabel(link.permissions))
                            }
                            .font(.system(size: 11))
                            .foregroundColor(OffriiTheme.textMuted)

                            // Creation + expiration
                            HStack(spacing: 4) {
                                Image(systemName: "clock")
                                    .font(.system(size: 9))
                                Text(link.createdAt, style: .relative)
                                if let exp = link.expiresAt {
                                    Text("·")
                                    Image(systemName: "timer")
                                        .font(.system(size: 9))
                                    Text(NSLocalizedString("share.expiresAt", comment: "") + " ")
                                        .font(.system(size: 10))
                                    + Text(exp, style: .relative)
                                        .font(.system(size: 10))
                                } else {
                                    Text("·")
                                    Text(NSLocalizedString("share.ttl.never", comment: ""))
                                }
                            }
                            .font(.system(size: 10))
                            .foregroundColor(OffriiTheme.textMuted)

                            // Item names for selection links
                            if let names = linkItemNames[link.id], !names.isEmpty {
                                Text(names.joined(separator: ", "))
                                    .font(.system(size: 11))
                                    .foregroundColor(OffriiTheme.textSecondary)
                                    .lineLimit(2)
                            }
                        }

                        // Actions: copy + delete
                        HStack(spacing: OffriiTheme.spacingBase) {
                            Spacer()

                            Button {
                                UIPasteboard.general.string = link.displayUrl
                                OffriiHaptics.success()
                                showToast(NSLocalizedString("share.linkCopied", comment: ""))
                            } label: {
                                Label(NSLocalizedString("share.copyLink", comment: ""), systemImage: "doc.on.doc")
                                    .font(.system(size: 11, weight: .medium))
                                    .foregroundColor(OffriiTheme.primary)
                            }

                            Button {
                                Task { await deleteLink(id: link.id) }
                            } label: {
                                Label(NSLocalizedString("common.delete", comment: ""), systemImage: "trash")
                                    .font(.system(size: 11, weight: .medium))
                                    .foregroundColor(OffriiTheme.danger)
                            }
                        }
                    }
                    .padding(OffriiTheme.spacingMD)
                    .background(isDisabled ? OffriiTheme.surface.opacity(0.6) : OffriiTheme.surface)
                    .cornerRadius(OffriiTheme.cornerRadiusMD)
                    .overlay(
                        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                            .strokeBorder(isDisabled ? OffriiTheme.border : .clear, lineWidth: 1)
                    )
                    .contextMenu {
                        Button {
                            editingLink = link
                        } label: {
                            Label(NSLocalizedString("wishlist.edit", comment: ""), systemImage: "pencil")
                        }

                        Button {
                            UIPasteboard.general.string = link.displayUrl
                            OffriiHaptics.success()
                            showToast(NSLocalizedString("share.linkCopied", comment: ""))
                        } label: {
                            Label(NSLocalizedString("share.copyLink", comment: ""), systemImage: "doc.on.doc")
                        }

                        Divider()

                        Button(role: .destructive) {
                            Task { await deleteLink(id: link.id) }
                        } label: {
                            Label(NSLocalizedString("common.delete", comment: ""), systemImage: "trash")
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingLG)
                }
            } else if isLoadingLinks {
                SkeletonRow()
                    .padding(.horizontal, OffriiTheme.spacingLG)
            }
        }
    }

    // MARK: - Scope Option

    private func scopeOption(_ option: ShareScope, icon: String, label: String) -> some View {
        Button {
            withAnimation(OffriiAnimation.snappy) {
                scope = option
            }
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

    // MARK: - Helpers

    private var isCreateDisabled: Bool {
        if scope == .category && selectedCategoryIds.isEmpty { return true }
        if scope == .selection && pickedItemIds.isEmpty { return true }
        return false
    }

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
        do { circles = try await CircleService.shared.listCircles() } catch {}
        isLoadingCircles = false
    }

    private func loadShareLinks() async {
        isLoadingLinks = true
        do { shareLinks = try await APIClient.shared.request(.listShareLinks) } catch {}
        isLoadingLinks = false
    }

    private func shareToCircle(_ circle: OffriiCircle) async {
        // For now, this is a placeholder — sharing all items to a circle
        // would require iterating items. Just mark as shared for UX feedback.
        _ = withAnimation {
            sharedCircleIds.insert(circle.id)
        }
        OffriiHaptics.success()
    }

    private func createLink(openShareSheet: Bool) async {
        isCreatingLink = true

        let scopeData: ScopeData?
        let scopeStr: String

        switch scope {
        case .all:
            scopeStr = "all"
            scopeData = nil
        case .category:
            if selectedCategoryIds.count == 1, let catId = selectedCategoryIds.first {
                // Single category → use native backend scope "category"
                scopeStr = "category"
                scopeData = ScopeData(categoryId: catId.uuidString, itemIds: nil)
            } else {
                // Multi-category → use "selection" with items from those categories
                let itemIds = items
                    .filter { item in
                        guard let catId = item.categoryId else { return false }
                        return selectedCategoryIds.contains(catId) && !item.isPrivate
                    }
                    .map { $0.id.uuidString }
                if itemIds.isEmpty {
                    OffriiHaptics.error()
                    isCreatingLink = false
                    return
                }
                scopeStr = "selection"
                scopeData = ScopeData(categoryId: nil, itemIds: itemIds)
            }
        case .selection:
            scopeStr = "selection"
            scopeData = ScopeData(categoryId: nil, itemIds: pickedItemIds.map { $0.uuidString })
        }

        let body = CreateShareLinkBody(
            expiresAt: linkTTL.expiresAt,
            label: linkLabel.trimmingCharacters(in: .whitespaces).isEmpty ? nil : linkLabel.trimmingCharacters(in: .whitespaces),
            permissions: permission,
            scope: scopeStr,
            scopeData: scopeData
        )

        do {
            let response: ShareLinkResponse = try await APIClient.shared.request(.createShareLink(body))
            let url = response.displayUrl

            UIPasteboard.general.string = url
            // Store display names for this link
            let names: [String]
            switch scope {
            case .selection:
                names = items.filter { pickedItemIds.contains($0.id) }.map(\.name)
            case .category:
                let catNames = categories
                    .filter { selectedCategoryIds.contains($0.id) }
                    .map(\.name)
                names = catNames
            case .all:
                names = []
            }
            linkItemNames[response.id] = names

            withAnimation(OffriiAnimation.defaultSpring) {
                justCreatedLink = url
                shareLinks.insert(response, at: 0)
            }
            OffriiHaptics.success()

            if openShareSheet {
                // Small delay to let the UI update before opening system share sheet
                try? await Task.sleep(for: .milliseconds(300))
                await MainActor.run {
                    if let url = URL(string: url) {
                        let activityVC = UIActivityViewController(activityItems: [url], applicationActivities: nil)
                        if let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
                           let rootVC = windowScene.windows.first?.rootViewController {
                            rootVC.present(activityVC, animated: true)
                        }
                    }
                }
            }
        } catch {
            OffriiHaptics.error()
        }
        isCreatingLink = false
    }

    private func deleteLink(id: UUID) async {
        do {
            try await APIClient.shared.requestVoid(.deleteShareLink(id: id))
            withAnimation {
                shareLinks.removeAll { $0.id == id }
            }
            OffriiHaptics.success()
            showToast(NSLocalizedString("share.linkDeleted", comment: ""))
        } catch {
            OffriiHaptics.error()
        }
    }

    private func showToast(_ message: String) {
        withAnimation(OffriiAnimation.defaultSpring) {
            toastMessage = message
        }
    }
}
