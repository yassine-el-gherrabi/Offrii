import SwiftUI

// MARK: - MemberCarousel

struct MemberCarousel: View {
    let members: [CircleMember]
    @Binding var selectedMemberId: UUID?
    let currentUserId: UUID?

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: OffriiTheme.spacingSM) {
                allButton

                ForEach(members) { member in
                    memberButton(member)
                }
            }
            .padding(.horizontal, OffriiTheme.spacingBase)
            .padding(.vertical, OffriiTheme.spacingSM)
        }
    }

    // MARK: - All Button

    private var allButton: some View {
        Button {
            OffriiHaptics.selection()
            selectedMemberId = nil
        } label: {
            VStack(spacing: 4) {
                Circle()
                    .fill(
                        LinearGradient(
                            colors: [OffriiTheme.primary.opacity(0.3), OffriiTheme.accent.opacity(0.2)],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .frame(width: 44, height: 44)
                    .overlay(
                        Image(systemName: "person.2.fill")
                            .font(.system(size: 16))
                            .foregroundColor(OffriiTheme.primary)
                    )
                    .overlay(
                        Circle()
                            .strokeBorder(
                                selectedMemberId == nil ? OffriiTheme.primary : .clear,
                                lineWidth: 2.5
                            )
                    )

                Text(NSLocalizedString("entraide.category.all", comment: ""))
                    .font(.system(size: 11, weight: selectedMemberId == nil ? .semibold : .regular))
                    .foregroundColor(selectedMemberId == nil ? OffriiTheme.primary : OffriiTheme.textMuted)
                    .lineLimit(1)
            }
            .frame(width: 56)
        }
        .buttonStyle(.plain)
    }

    // MARK: - Member Button

    private func memberButton(_ member: CircleMember) -> some View {
        let isSelected = selectedMemberId == member.userId
        let isMe = member.userId == currentUserId
        let displayLabel = isMe
            ? NSLocalizedString("circles.detail.myWishes", comment: "")
            : (member.displayName ?? member.username)

        return Button {
            OffriiHaptics.selection()
            if isSelected {
                // Tap same = deselect → back to "all"
                selectedMemberId = nil
            } else {
                // Tap different = select this one
                selectedMemberId = member.userId
            }
        } label: {
            VStack(spacing: 4) {
                AvatarView(member.displayName ?? member.username, size: .medium)
                    .overlay(
                        Circle()
                            .strokeBorder(
                                isSelected ? OffriiTheme.primary : .clear,
                                lineWidth: 2.5
                            )
                    )

                Text(displayLabel)
                    .font(.system(size: 11, weight: isSelected ? .semibold : .regular))
                    .foregroundColor(isSelected ? OffriiTheme.primary : OffriiTheme.textMuted)
                    .lineLimit(1)
            }
            .frame(width: 56)
        }
        .buttonStyle(.plain)
    }
}
