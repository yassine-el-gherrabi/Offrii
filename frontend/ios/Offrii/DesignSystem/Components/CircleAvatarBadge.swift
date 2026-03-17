import NukeUI
import SwiftUI

/// Reusable circle avatar with type badge (person/person.2) overlay.
/// Used in ItemDetailSheet, ItemEditView, and anywhere shared circles are displayed.
struct CircleAvatarBadge: View {
    let circle: SharedCircleInfo
    var size: CGFloat = 28
    var fontSize: CGFloat = 12

    var body: some View {
        ZStack {
            if let url = circle.imageURL {
                LazyImage(url: url) { state in
                    if let image = state.image {
                        image
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                            .frame(width: size, height: size)
                            .clipShape(Circle())
                    } else {
                        initialsView
                    }
                }
            } else {
                initialsView
            }
        }
        .overlay(alignment: .bottomTrailing) {
            Image(systemName: circle.isDirect == true ? "person.fill" : "person.2.fill")
                .font(.system(size: size * 0.25, weight: .bold))
                .foregroundColor(.white)
                .padding(2)
                .background(OffriiTheme.primary)
                .clipShape(Circle())
                .overlay(Circle().strokeBorder(.white, lineWidth: 1))
                .offset(x: size * 0.1, y: size * 0.1)
        }
    }

    private var initialsView: some View {
        Text(circle.initial)
            .font(.system(size: fontSize, weight: .bold))
            .foregroundColor(.white)
            .frame(width: size, height: size)
            .background(OffriiTheme.primary)
            .clipShape(Circle())
    }
}
